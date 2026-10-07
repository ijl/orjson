// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2020-2026)

use crate::serialize::{
    datetime::{write_date, write_datetime, write_time},
    error::SerializeError,
    fragment::serialize_fragment,
    non_str::pyobject_to_string,
    num::{serialize_float, serialize_int},
    numpy::{NumpyArray, NumpyScalar, PyArrayError},
    obtype::ObType,
    obtype::{pyobject_to_obtype_likely, pyobject_to_obtype_unlikely},
    state::SerializerState,
    uuid::write_uuid,
    writer::{BytesWriter, JsonWriter, WriteFormatter, format_escaped_str},
};
use crate::typeref::{
    DATACLASS_FIELDS_STR, DICT_STR, FIELD_TYPE, FIELD_TYPE_STR, SLOTS_STR, TRUE, VALUE_STR,
};

use crate::opt::{
    APPEND_NEWLINE, NON_STR_KEYS, NOT_PASSTHROUGH, PASSTHROUGH_DATETIME, SORT_KEYS,
    SORT_OR_NON_STR_KEYS, STRICT_INTEGER,
};
use core::ptr::NonNull;

use crate::ffi::{
    BorrowedPyRef, Py_DECREF, Py_REFCNT, PyBoolRef, PyDateRef, PyDateTimeRef, PyDictIterator,
    PyDictRef, PyErr_Clear, PyFloatRef, PyFragmentRef, PyIntRef, PyListIterator, PyListRef,
    PyObject, PyObject_GetAttr, PyObject_Type, PyObject_Vectorcall, PyStrRef, PyStrSubclassRef,
    PyTimeRef, PyTupleIterator, PyTupleRef, PyTypeRef, PyUuidRef, PyVectorcall_NARGS,
};

const PER_ITEM_RESERVE: usize = 128;

#[cold]
#[inline(never)]
pub(crate) fn format_default_unserializable(ptr: *mut PyObject) -> PyStrRef {
    unsafe {
        let mut msg = String::from("Type is not JSON serializable: ");
        let name = core::ffi::CStr::from_ptr((*PyObject_Type(ptr)).tp_name).to_string_lossy();
        msg.push_str(name.as_ref());

        PyStrRef::from_str(&msg)
    }
}

enum DictOption {
    Generic,
    NonStr,
    Sorted,
}

impl DictOption {
    fn from_state(state: SerializerState) -> Self {
        if state.is_disabled(SORT_OR_NON_STR_KEYS) {
            DictOption::Generic
        } else if opt_enabled!(state.opts(), NON_STR_KEYS) {
            DictOption::NonStr
        } else {
            DictOption::Sorted
        }
    }
}

#[repr(align(128))]
pub(crate) struct ContainerSerializer<F> {
    writer: BytesWriter,
    formatter: F,
    recursion: u32,
    default_calls: u32,
    state: SerializerState,
    dict_opt: DictOption,
    default: Option<NonNull<PyObject>>,
}

impl<F> ContainerSerializer<F>
where
    F: WriteFormatter,
{
    pub fn new(
        formatter: F,
        state: SerializerState,
        default: Option<NonNull<PyObject>>,
    ) -> ContainerSerializer<F> {
        ContainerSerializer::<F> {
            formatter: formatter,
            writer: BytesWriter::new(),
            state: state,
            dict_opt: DictOption::from_state(state),
            recursion: 0,
            default_calls: 0,
            default: default,
        }
    }
}

impl<F> ContainerSerializer<F>
where
    F: WriteFormatter + Clone,
{
    pub fn write(&mut self, ptr: *mut PyObject) -> Result<NonNull<PyObject>, SerializeError> {
        match self.serialize(ptr) {
            Ok(()) => Ok(self
                .writer
                .finish(opt_enabled!(self.state.opts(), APPEND_NEWLINE))),
            Err(err) => {
                cold_path!();
                self.writer.abort();
                Err(err)
            }
        }
    }

    #[inline(always)]
    fn serialize_pair(&mut self, key: &str, value: *mut PyObject) -> Result<(), SerializeError> {
        self.write_escaped_str(key);
        F::map_key_value_separator(&mut self.writer);
        self.serialize(value)
    }

    #[inline(always)]
    fn serialize_tuple(&mut self, ob: PyTupleRef) -> Result<(), SerializeError> {
        if ob.len() == 0 {
            self.writer.reserve_minimum();
            self.writer.put_slice(b"[]");
            Ok(())
        } else {
            let mut array_iter = PyTupleIterator::from_tuple(ob).fuse();
            self.serialize_array_inner(&mut array_iter)
        }
    }

    #[inline(always)]
    fn serialize_list(&mut self, ob: PyListRef) -> Result<(), SerializeError> {
        if self.recursion == 254 {
            cold_path!();
            return Err(SerializeError::RecursionLimit);
        }
        if ob.len() == 0 {
            self.writer.reserve_minimum();
            self.writer.put_slice(b"[]");
            Ok(())
        } else {
            let mut array_iter = PyListIterator::from_list(ob).fuse();
            self.serialize_array_inner(&mut array_iter)
        }
    }

    #[inline(never)]
    fn serialize_array_inner<T: Iterator<Item = BorrowedPyRef> + ExactSizeIterator>(
        &mut self,
        array_iter: &mut T,
    ) -> Result<(), SerializeError> {
        self.recursion += 1;
        assume!(array_iter.len() > 0);
        F::reserve_array(&mut self.writer, array_iter.len(), PER_ITEM_RESERVE);
        F::array_open(&mut self.writer);

        self.serialize(array_iter.next().unwrap().as_ptr())?;

        for each in array_iter {
            F::item_separator(&mut self.writer);
            self.serialize(each.as_ptr())?;
        }

        F::array_close(&mut self.writer);
        self.recursion -= 1;
        Ok(())
    }

    #[inline(always)]
    fn serialize_map(&mut self, dict: PyDictRef) -> Result<(), SerializeError> {
        if self.recursion == 254 {
            cold_path!();
            return Err(SerializeError::RecursionLimit);
        }
        if dict.len() == 0 {
            self.writer.reserve_minimum();
            self.writer.put_slice(b"{}");
            Ok(())
        } else {
            self.recursion += 1;
            let ret = match self.dict_opt {
                DictOption::Generic => self.serialize_dict_generic(dict),
                DictOption::NonStr => self.serialize_dict_nonstr(dict),
                DictOption::Sorted => self.serialize_dict_sorted(dict),
            };
            self.recursion -= 1;
            ret
        }
    }

    #[inline(never)]
    fn serialize_dict_generic(&mut self, dict: PyDictRef) -> Result<(), SerializeError> {
        F::map_open(&mut self.writer);

        let map_iter = PyDictIterator::from_dict(dict).fuse();
        F::reserve_map(&mut self.writer, map_iter.len(), PER_ITEM_RESERVE);
        let mut repeating = false;

        for (key, value) in map_iter {
            if repeating {
                F::item_separator(&mut self.writer);
            } else {
                repeating = true;
            }
            let ptr = value.as_ptr();
            match PyStrRef::from_ptr(key.as_ptr()) {
                Ok(val) => self.serialize_str(val)?,
                Err(_) => {
                    cold_path!();
                    return Err(SerializeError::KeyMustBeStr);
                }
            }
            F::map_key_value_separator(&mut self.writer);
            self.serialize(ptr)?;
        }

        F::map_close(&mut self.writer);
        Ok(())
    }

    #[inline(never)]
    fn serialize_str_value_items(
        &mut self,
        items: &Vec<(&str, *mut PyObject)>,
    ) -> Result<(), SerializeError> {
        F::map_open(&mut self.writer);
        F::reserve_map(&mut self.writer, items.len(), PER_ITEM_RESERVE);
        let mut repeating = false;
        for (key, val) in items.iter().fuse() {
            if repeating {
                F::item_separator(&mut self.writer);
            } else {
                repeating = true;
            }
            self.serialize_pair(key, *val)?;
        }

        F::map_close(&mut self.writer);
        Ok(())
    }

    #[inline(never)]
    fn serialize_dict_nonstr(&mut self, dict: PyDictRef) -> Result<(), SerializeError> {
        let opts = self.state.opts() & NOT_PASSTHROUGH;

        let map_iter = PyDictIterator::from_dict(dict).fuse();
        let mut items: Vec<(String, *mut PyObject)> = Vec::with_capacity(map_iter.len());
        for (key, value) in map_iter {
            if let Ok(ob) = PyStrRef::from_ptr(key.as_ptr()) {
                match ob.as_str() {
                    Some(uni) => {
                        items.push((String::from(uni), value.as_ptr()));
                    }
                    None => {
                        cold_path!();
                        return Err(SerializeError::InvalidStr);
                    }
                }
            } else {
                match pyobject_to_string(key.as_ptr(), opts) {
                    Ok(key_as_str) => items.push((key_as_str, value.as_ptr())),
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
        }

        let mut items_as_str: Vec<(&str, *mut PyObject)> = Vec::with_capacity(items.len());
        items
            .iter()
            .fuse()
            .for_each(|(key, val)| items_as_str.push(((*key).as_str(), *val)));

        if opt_enabled!(opts, SORT_KEYS) {
            sort_dict_items(&mut items_as_str);
        }

        self.serialize_str_value_items(&items_as_str)?;
        Ok(())
    }

    #[inline(never)]
    fn serialize_dict_sorted(&mut self, dict: PyDictRef) -> Result<(), SerializeError> {
        let map_iter = PyDictIterator::from_dict(dict).fuse();
        let mut items: Vec<(&str, *mut PyObject)> = Vec::with_capacity(map_iter.len());
        for (key, value) in map_iter {
            // key
            match PyStrRef::from_ptr(key.as_ptr())
                .map_err(|_| {
                    cold_path!();
                    SerializeError::KeyMustBeStr
                })?
                .as_str()
            {
                None => {
                    cold_path!();
                    return Err(SerializeError::InvalidStr);
                }
                Some(uni) => {
                    items.push((uni, value.as_ptr()));
                }
            }
        }

        sort_dict_items(&mut items);

        self.serialize_str_value_items(&items)?;
        Ok(())
    }

    #[inline(never)]
    fn serialize_dataclass(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        self.recursion += 1;
        if self.recursion == 255 {
            cold_path!();
            return Err(SerializeError::RecursionLimit);
        }
        self.writer.reserve_minimum();
        let dict = unsafe { PyObject_GetAttr(ptr, DICT_STR) };
        let ob_type = unsafe { PyObject_Type(ptr) };
        if dict.is_null() {
            cold_path!();
            unsafe {
                PyErr_Clear();
            }
            let res = self.serialize_dataclass_inner_fallback(ptr);
            self.recursion -= 1;
            res
        } else if pydict_contains!(ob_type, SLOTS_STR) {
            let res = self.serialize_dataclass_inner_fallback(ptr);
            unsafe {
                Py_DECREF(dict);
            }
            self.recursion -= 1;
            res
        } else {
            let res =
                self.serialize_dataclass_inner_fast(unsafe { PyDictRef::from_ptr_unchecked(dict) });
            unsafe {
                Py_DECREF(dict);
            }
            self.recursion -= 1;
            res
        }
    }

    #[inline(never)]
    fn serialize_dataclass_inner_fast(&mut self, dict: PyDictRef) -> Result<(), SerializeError> {
        if dict.len() == 0 {
            cold_path!();
            self.writer.put_slice(b"{}");
            return Ok(());
        }

        F::map_open(&mut self.writer);

        let mut repeating = false;

        let map_iter = PyDictIterator::from_dict(dict).fuse();
        F::reserve_map(&mut self.writer, map_iter.len(), PER_ITEM_RESERVE);
        for (key, value) in map_iter {
            // key
            match PyStrRef::from_ptr(key.as_ptr())
                .map_err(|_| {
                    cold_path!();
                    SerializeError::KeyMustBeStr
                })?
                .as_str()
            {
                None => {
                    cold_path!();
                    return Err(SerializeError::InvalidStr);
                }
                Some(uni) => {
                    // can't have dataclass member name of empty str
                    if *uni.as_bytes().first().unwrap() == b'_' {
                        continue;
                    }

                    if repeating {
                        F::item_separator(&mut self.writer);
                    } else {
                        repeating = true;
                    }
                    self.write_escaped_str(uni);
                }
            }
            F::map_key_value_separator(&mut self.writer);
            self.serialize(value.as_ptr())?;
        }

        F::map_close(&mut self.writer);
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn serialize_dataclass_inner_fallback(
        &mut self,
        ptr: *mut PyObject,
    ) -> Result<(), SerializeError> {
        let fields =
            unsafe { PyDictRef::from_ptr_unchecked(PyObject_GetAttr(ptr, DATACLASS_FIELDS_STR)) };
        unsafe {
            Py_DECREF(fields.as_ptr());
        }

        let map_iter = PyDictIterator::from_dict(fields).fuse();
        if map_iter.len() == 0 {
            cold_path!();
            self.writer.put_slice(b"{}");
            return Ok(());
        }
        F::reserve_map(&mut self.writer, map_iter.len(), PER_ITEM_RESERVE);
        F::map_open(&mut self.writer);

        let mut repeating = false;
        for (attr, field) in map_iter {
            let field_type = unsafe { PyObject_GetAttr(field.as_ptr(), FIELD_TYPE_STR) };
            unsafe {
                Py_DECREF(field_type);
            }

            if unsafe { !core::ptr::eq(field_type.cast::<crate::ffi::PyTypeObject>(), FIELD_TYPE) }
            {
                continue;
            }

            let key_as_str = match unsafe { PyStrRef::from_ptr_unchecked(attr.as_ptr()).as_str() } {
                Some(uni) => uni,
                None => return Err(SerializeError::InvalidStr),
            };
            if key_as_str.as_bytes()[0] == b'_' {
                continue;
            }

            let value = unsafe { PyObject_GetAttr(ptr, attr.as_ptr()) };
            debug_assert!(unsafe { Py_REFCNT(value) >= 2 });
            unsafe {
                Py_DECREF(value);
            }

            if repeating {
                F::item_separator(&mut self.writer);
            } else {
                repeating = true;
            }
            self.serialize_pair(key_as_str, value)?;
        }

        F::map_close(&mut self.writer);
        Ok(())
    }

    #[cold]
    #[inline(never)]
    fn serialize_unknown(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        match self.default {
            Some(callable) => {
                if self.default_calls == 255 {
                    cold_path!();
                    return Err(SerializeError::DefaultRecursionLimit);
                }
                self.default_calls += 1;
                #[allow(clippy::cast_sign_loss)]
                let nargs = unsafe { PyVectorcall_NARGS(1) as usize };
                let default_obj = unsafe {
                    PyObject_Vectorcall(
                        callable.as_ptr(),
                        &raw const ptr,
                        nargs,
                        core::ptr::null_mut(),
                    )
                };
                if default_obj.is_null() {
                    cold_path!();
                    Err(SerializeError::UnsupportedType(
                        format_default_unserializable(ptr),
                    ))
                } else {
                    let res = self.serialize(default_obj);
                    unsafe {
                        Py_DECREF(default_obj);
                    }
                    self.default_calls -= 1;
                    res
                }
            }
            None => Err(SerializeError::UnsupportedType(
                format_default_unserializable(ptr),
            )),
        }
    }

    #[cold]
    #[inline(never)]
    fn serialize_numpy_array(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        match NumpyArray::new(self.formatter.clone(), ptr, self.state.opts()) {
            Ok(serializer) => serializer.write(&mut self.writer),
            Err(PyArrayError::Malformed) => Err(SerializeError::NumpyMalformed),
            Err(PyArrayError::NotContiguous | PyArrayError::UnsupportedDataType)
                if self.default.is_some() =>
            {
                self.serialize_unknown(ptr)
            }
            Err(PyArrayError::NotContiguous) => Err(SerializeError::NumpyNotCContiguous),
            Err(PyArrayError::NotNativeEndian) => Err(SerializeError::NumpyNotNativeEndian),
            Err(PyArrayError::UnsupportedDataType) => Err(SerializeError::NumpyUnsupportedDatatype),
        }
    }

    #[cold]
    #[inline(never)]
    fn serialize_numpy_scalar(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        NumpyScalar::new(ptr, self.state.opts()).write(&mut self.writer)
    }

    #[inline(always)]
    fn serialize_str(&mut self, ob: PyStrRef) -> Result<(), SerializeError> {
        {
            match ob.as_str() {
                Some(uni) => {
                    self.write_escaped_str(uni);
                    Ok(())
                }
                None => {
                    cold_path!();
                    Err(SerializeError::InvalidStr)
                }
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn serialize_str_subclass(&mut self, ob: PyStrSubclassRef) -> Result<(), SerializeError> {
        {
            match ob.as_str() {
                Some(uni) => {
                    self.write_escaped_str(uni);
                    Ok(())
                }
                None => {
                    cold_path!();
                    Err(SerializeError::InvalidStr)
                }
            }
        }
    }

    #[inline(always)]
    fn write_escaped_str(&mut self, val: &str) {
        unsafe {
            self.writer.reserve(val.len() * 8 + 32);
            let written = format_escaped_str(&mut self.writer, val);
            self.writer.advance_mut(written);
        }
    }

    #[inline(always)]
    fn serialize_none(&mut self) {
        self.writer.put_null();
    }

    #[inline(always)]
    fn serialize_bool(&mut self, ob: PyBoolRef) {
        self.writer
            .put_bool(unsafe { core::ptr::eq(ob.as_ptr(), TRUE) })
    }

    #[cold]
    #[inline(never)]
    fn serialize_uuid(&mut self, ob: PyUuidRef) {
        self.writer.quote();
        write_uuid(ob, &mut self.writer);
        self.writer.quote();
    }

    #[inline(never)]
    fn serialize_datetime(&mut self, ob: PyDateTimeRef) -> Result<(), SerializeError> {
        if self.state.is_enabled(PASSTHROUGH_DATETIME) {
            cold_path!();
            self.serialize_unknown(ob.as_ptr())
        } else {
            self.writer.quote();
            write_datetime(ob, self.state.opts(), &mut self.writer).map_err(|_| {
                cold_path!();
                SerializeError::DatetimeLibraryUnsupported
            })?;
            self.writer.quote();
            Ok(())
        }
    }

    #[cold]
    #[inline(never)]
    fn serialize_date(&mut self, ob: PyDateRef) -> Result<(), SerializeError> {
        if self.state.is_enabled(PASSTHROUGH_DATETIME) {
            cold_path!();
            self.serialize_unknown(ob.as_ptr())
        } else {
            self.writer.quote();
            write_date(ob, &mut self.writer);
            self.writer.quote();
            Ok(())
        }
    }

    #[cold]
    #[inline(never)]
    fn serialize_time(&mut self, ob: PyTimeRef) -> Result<(), SerializeError> {
        if self.state.is_enabled(PASSTHROUGH_DATETIME) {
            cold_path!();
            self.serialize_unknown(ob.as_ptr())
        } else {
            self.writer.quote();
            write_time(ob, self.state.opts(), &mut self.writer)?;
            self.writer.quote();
            Ok(())
        }
    }

    #[cold]
    #[inline(never)]
    fn serialize_enum(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        let value = unsafe { PyObject_GetAttr(ptr, VALUE_STR) };
        debug_assert!(unsafe { Py_REFCNT(value) >= 2 });
        self.serialize(value)
    }

    #[inline(always)]
    pub fn serialize(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        match pyobject_to_obtype_likely(PyTypeRef::from_pyobject(ptr)) {
            Some(ObType::Str) => {
                self.serialize_str(unsafe { PyStrRef::from_ptr_unchecked(ptr) })?
            }
            Some(ObType::Int) => serialize_int(
                &mut self.writer,
                unsafe { PyIntRef::from_ptr_unchecked(ptr) },
                self.state.is_enabled(STRICT_INTEGER),
            )?,
            Some(ObType::Bool) => {
                self.serialize_bool(unsafe { PyBoolRef::from_ptr_unchecked(ptr) })
            }
            Some(ObType::None) => self.serialize_none(),
            Some(ObType::Float) => serialize_float(&mut self.writer, unsafe {
                PyFloatRef::from_ptr_unchecked(ptr)
            }),
            Some(ObType::List) => {
                self.serialize_list(unsafe { PyListRef::from_ptr_unchecked(ptr) })?
            }
            Some(ObType::Dict) => {
                self.serialize_map(unsafe { PyDictRef::from_ptr_unchecked(ptr) })?
            }
            Some(ObType::Datetime) => {
                self.serialize_datetime(unsafe { PyDateTimeRef::from_ptr_unchecked(ptr) })?;
            }
            Some(_) => unreachable_unchecked!(),
            None => {
                cold_path!();
                self.serialize_unlikely(ptr)?
            }
        }
        Ok(())
    }

    #[cold]
    #[inline(never)]
    pub fn serialize_unlikely(&mut self, ptr: *mut PyObject) -> Result<(), SerializeError> {
        match pyobject_to_obtype_unlikely(PyTypeRef::from_pyobject(ptr), self.state.opts()) {
            ObType::List => self.serialize_list(unsafe { PyListRef::from_ptr_unchecked(ptr) })?,
            ObType::Dict => self.serialize_map(unsafe { PyDictRef::from_ptr_unchecked(ptr) })?,
            ObType::Int => serialize_int(
                &mut self.writer,
                unsafe { PyIntRef::from_ptr_unchecked(ptr) },
                self.state.is_enabled(STRICT_INTEGER),
            )?,
            ObType::Datetime => {
                self.serialize_datetime(unsafe { PyDateTimeRef::from_ptr_unchecked(ptr) })?;
            }
            ObType::Uuid => self.serialize_uuid(unsafe { PyUuidRef::from_ptr_unchecked(ptr) }),
            ObType::Date => self.serialize_date(unsafe { PyDateRef::from_ptr_unchecked(ptr) })?,
            ObType::Time => self.serialize_time(unsafe { PyTimeRef::from_ptr_unchecked(ptr) })?,
            ObType::StrSubclass => {
                cold_path!();
                self.serialize_str_subclass(unsafe { PyStrSubclassRef::from_ptr_unchecked(ptr) })?;
            }
            ObType::Float => {
                serialize_float(&mut self.writer, unsafe {
                    PyFloatRef::from_ptr_unchecked(ptr)
                });
            }
            ObType::Fragment => {
                cold_path!();
                serialize_fragment(&mut self.writer, unsafe {
                    PyFragmentRef::from_ptr_unchecked(ptr)
                })?;
            }
            ObType::Unknown => {
                cold_path!();
                self.serialize_unknown(ptr)?;
            }
            ObType::Tuple => {
                self.serialize_tuple(unsafe { PyTupleRef::from_ptr_unchecked(ptr) })?
            }
            ObType::Dataclass => self.serialize_dataclass(ptr)?,
            ObType::NumpyScalar => {
                cold_path!();
                self.serialize_numpy_scalar(ptr)?;
            }
            ObType::NumpyArray => {
                cold_path!();
                self.serialize_numpy_array(ptr)?;
            }
            ObType::Enum => self.serialize_enum(ptr)?,
            ObType::None | ObType::Bool | ObType::Str => unreachable_unchecked!(),
        }
        Ok(())
    }
}

#[inline(never)]
fn sort_dict_items(items: &mut Vec<(&str, *mut PyObject)>) {
    items.sort_unstable_by(|a, b| a.0.cmp(b.0));
}
