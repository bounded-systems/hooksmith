# Hooks Directory

This directory contains hook scripts that can be built and managed by the hooksmith CLI.

## Usage

Hook scripts placed in this directory can be:
- Built into binaries using `hooksmith build`
- Installed using `hooksmith install`
- Listed using `hooksmith list`

## Structure

```
hooks/
├── README.md          # This file
└── [hook-scripts]     # Hook scripts to be built
```

## Example

Place your hook scripts here and they will be available for building and installation through the hooksmith CLI. 
