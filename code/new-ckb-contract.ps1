<#
.SYNOPSIS
    Scaffold a new CKB smart-contract (on-chain script) project, mirroring the
    hand-rolled hello_world layout (ckb-std + custom linker script, no capsule).

.EXAMPLE
    .\new-ckb-contract.ps1 my_contract
    .\new-ckb-contract.ps1 my_contract -Build
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Name,

    # ckb-std version to pin in Cargo.toml
    [string]$CkbStdVersion = "0.16",

    # Pass -Build to run `cargo build --release` after scaffolding
    [switch]$Build
)

$ErrorActionPreference = "Stop"

# Windows PowerShell's `Out-File -Encoding utf8` prepends a UTF-8 BOM, which
# rust-lld rejects in the linker script ("unknown directive"). Write UTF-8
# without a BOM instead.
function Set-NoBom {
    param(
        [Parameter(ValueFromPipeline = $true)][string]$Content,
        [Parameter(Position = 0, Mandatory = $true)][string]$Path
    )
    process {
        [System.IO.File]::WriteAllText($Path, $Content, (New-Object System.Text.UTF8Encoding $false))
    }
}

# Crate names must be valid Rust identifiers; normalise dashes -> underscores.
$crate = $Name -replace '-', '_'
if ($crate -notmatch '^[a-zA-Z][a-zA-Z0-9_]*$') {
    throw "Invalid project name '$Name'. Use letters, digits, and underscores; must start with a letter."
}

# Create the project inside the same folder this script lives in (code/).
$root = Join-Path $PSScriptRoot $Name
if (Test-Path $root) {
    throw "Directory already exists: $root"
}

Write-Host "Creating CKB contract '$crate' at $root" -ForegroundColor Cyan

New-Item -ItemType Directory -Path $root            | Out-Null
New-Item -ItemType Directory -Path "$root\.cargo"   | Out-Null
New-Item -ItemType Directory -Path "$root\src"      | Out-Null

# --- .cargo/config.toml ------------------------------------------------------
@'
[build]
target = "riscv64imac-unknown-none-elf"

[target.riscv64imac-unknown-none-elf]
rustflags = [
    "-C", "link-arg=-Tckb-std.ld",
]
'@ | Set-NoBom "$root\.cargo\config.toml"

# --- .gitignore --------------------------------------------------------------
@'
/target
'@ | Set-NoBom "$root\.gitignore"

# --- Cargo.toml --------------------------------------------------------------
@"
[package]
name = "$crate"
version = "0.1.0"
edition = "2021"

[dependencies]
ckb-std = { version = "$CkbStdVersion", default-features = false, features = ["allocator", "dummy-atomic"] }

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = false
"@ | Set-NoBom "$root\Cargo.toml"

# --- ckb-std.ld (linker script, fixed template) ------------------------------
@'
ENTRY(_start)

PHDRS {
    code PT_LOAD FLAGS(5);  /* R + E */
    data PT_LOAD FLAGS(6);  /* R + W */
}

SECTIONS {
    . = 0x10000;

    .text : {
        KEEP(*(.text._start))
        *(.text .text.*)
    } :code

    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } :code

    . = ALIGN(0x1000);

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.note .note.*)
        *(.riscv.attributes)
    }
}
'@ | Set-NoBom "$root\ckb-std.ld"

# --- src/main.rs -------------------------------------------------------------
@'
#![no_std]
#![no_main]
#![allow(unexpected_cfgs)]

use ckb_std::debug;

ckb_std::entry!(program_entry);
ckb_std::default_alloc!();

pub fn program_entry() -> i8 {
    debug!("Hello from CRATE_NAME!");
    0
}
'@.Replace('CRATE_NAME', $crate) | Set-NoBom "$root\src\main.rs"

Write-Host "Created:" -ForegroundColor Green
Get-ChildItem -Recurse -File $root | ForEach-Object {
    Write-Host ("  " + $_.FullName.Substring($root.Length + 1))
}

if ($Build) {
    Write-Host "`nBuilding (cargo build --release)..." -ForegroundColor Cyan
    Push-Location $root
    try {
        cargo build --release
    } finally {
        Pop-Location
    }
    $bin = "$root\target\riscv64imac-unknown-none-elf\release\$crate"
    if (Test-Path $bin) {
        $size = (Get-Item $bin).Length
        Write-Host "`nBinary: $bin ($size bytes)" -ForegroundColor Green
    }
} else {
    Write-Host "`nNext steps:" -ForegroundColor Yellow
    Write-Host "  cd `"$root`""
    Write-Host "  cargo build --release"
    Write-Host "  # binary -> target\riscv64imac-unknown-none-elf\release\$crate"
}
