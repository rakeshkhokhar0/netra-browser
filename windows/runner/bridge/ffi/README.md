# Windows Rust FFI Bridge

This folder owns the narrow C++ to Rust bridge used by the Windows native
pipeline.

## Files

- `rust_bridge.h`: public bridge contract used by native request filtering.
- `rust_bridge.cpp`: implementation of the bridge contract.

## What This Folder Does

- Accepts request metadata from the native WebView2 interception layer.
- Forwards `url` and `resource_type` values to Rust-owned filtering logic.
- Returns a boolean allow/block decision to the caller in native C++.

## Work Done

- Stable function boundary (`ShouldBlockRequest`) is defined and documented.
- Bridge implementation is wired and callable from request interception code.
- Clear ownership is set: C++ only translates values, Rust owns policy logic.

## Current Limitation

- Current implementation is intentionally stubbed and always returns `false`.
- Real filtering decision logic will be completed when Rust request pipeline
  integration is finalized.
