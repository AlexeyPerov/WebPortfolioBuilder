//! Core engine for Portfolio Website Builder (Phase 1 implementation).
//!
//! See [README.md](../README.md) for template engine choice and public API plan.

#![allow(dead_code)]

/// Non-fatal configuration issue (mirrors Go `ConfigWarning`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    pub path: String,
    pub message: String,
}

// Phase 1.1+: load_site_bundle, validate_site_bundle, render_site_bundle
