#!/usr/bin/env bash

# Function to prompt for the GitHub personal access token
prompt_for_token() {
    echo "Please generate a GitHub Personal Access Token with the following scopes:"
    echo "  - read:packages"
    echo "Visit: https://github.com/settings/tokens"
    echo "Once generated, enter the token below."

    read -s -p "GitHub Personal Access Token: " GITHUB_TOKEN
    echo

    # Validate the input
    if [ -z "$GITHUB_TOKEN" ]; then
        echo "Error: GitHub Personal Access Token is required."
        exit 1
    fi

    # GitHub username
    read -p "Enter your GitHub username: " GITHUB_USERNAME

    if [ -z "$GITHUB_USERNAME" ]; then
        echo "Error: GitHub username is required."
        exit 1
    fi

    echo "$GITHUB_TOKEN" | docker login ghcr.io -u "$GITHUB_USERNAME" --password-stdin
}

# Check if the Docker Auth is enabled.
if grep -q "ghcr.io" "$HOME/.docker/config.json";
then
    echo "🐋 Docker Auth enabled."
else
    prompt_for_token
fi

echo "📦 Starting the Container..."