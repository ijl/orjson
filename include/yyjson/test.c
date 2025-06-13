#include "yyjson.h"
#include <stdio.h>

int main() {
    // JSON string with a big number
    const char *json = "{\"big_num\": 123456789012345678901234567890}";

    // Parse JSON with YYJSON_READ_BIGNUM_AS_RAW flag
    yyjson_doc *doc = yyjson_read_opts(json, strlen(json), YYJSON_READ_BIGNUM_AS_RAW, NULL, NULL);
    if (!doc) {
        fprintf(stderr, "Failed to parse JSON\n");
        return 1;
    }

    // // Get the root object
    // yyjson_val *root = yyjson_doc_get_root(doc);
    // if (!root) {
    //     fprintf(stderr, "Failed to get root object\n");
    //     yyjson_doc_free(doc);
    //     return 1;
    // }

    // // Get the value of "big_num"
    // yyjson_val *big_num = yyjson_obj_get(root, "big_num");
    // if (!big_num) {
    //     fprintf(stderr, "Failed to get big_num\n");
    //     yyjson_doc_free(doc);
    //     return 1;
    // }

    // Print the entire JSON document
    const char *json_dump = yyjson_write(doc, YYJSON_WRITE_PRETTY, NULL);
    if (!json_dump) {
        fprintf(stderr, "Failed to dump JSON document\n");
        yyjson_doc_free(doc);
        return 1;
    }

    printf("JSON Document:\n%s\n", json_dump);

    // Free the dumped JSON string
    free((void *)json_dump);

    

}
