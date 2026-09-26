//! 内建插件
//! 
//! 后续可以把 search 模块分离，然后内置多插件
//! - 应用插件： UWP / 开始菜单 / 注册表 （可先用 scan 类型代替）
//! - 浏览器书签插件（可先用 web 类型代替）
//! - 进程杀手插件（可先用 cmd 类型代替）

pub mod base;
pub mod common;

pub mod action_systems;

pub mod action;

pub mod persistence;

pub mod search;

pub mod stat;

pub use stat::{load_stat, reload_setting};
