#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
target_root="${HOME}/.vscode/extensions"
target_dir="${target_root}/cradle.highlight"

if [[ ! -d "${target_root}" ]]; then
	printf 'VS Code extensions directory not found: %s\n' "${target_root}" >&2
	exit 1
fi

rm -rf "${target_dir}"
mkdir -p "${target_dir}"
cp -a "${script_dir}/." "${target_dir}/"

printf 'Installed cradle.highlight to %s\n' "${target_dir}"
