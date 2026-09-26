//! 内建插件
//! 
//! todo 把 search 模块分离
//! 
//! todo 默认 scan 或独立应用插件： UWP / 开始菜单 / 注册表
//! 
//! todo 插件：浏览器书签
//! 
//! todo 插件：进程列表和 kill

pub mod base;
pub mod common;

pub mod action_systems;

pub mod action;

pub mod persistence;

pub mod search;

pub mod stat;

pub use stat::{load_stat, reload_setting};
