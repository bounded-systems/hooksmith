use hooksmith::{
    log_header, log_info, log_success, AttributeFilterType, ConfigHookType, CustomRefType,
    ExternalToolType, GitHookSurface, LfsHookType, LifecycleHookType, NotesHookType,
    RuntimeProxyType, WorktreeHookType,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("GIT HOOK SURFACE DEMO");
    println!();

    // Demonstrate all hook surface types
    demonstrate_lifecycle_hooks()?;
    demonstrate_lfs_hooks()?;
    demonstrate_attribute_filters()?;
    demonstrate_config_hooks()?;
    demonstrate_notes_hooks()?;
    demonstrate_custom_refs()?;
    demonstrate_worktree_hooks()?;
    demonstrate_external_tools()?;
    demonstrate_runtime_proxies()?;

    log_success("Git Hook Surface demo completed!");
    Ok(())
}

fn demonstrate_lifecycle_hooks() -> Result<(), Box<dyn std::error::Error>> {
    log_header("LIFECYCLE HOOKS");
    println!();

    let hooks = vec![
        GitHookSurface::LifecycleHook(LifecycleHookType::PreCommit),
        GitHookSurface::LifecycleHook(LifecycleHookType::CommitMsg),
        GitHookSurface::LifecycleHook(LifecycleHookType::PrePush),
        GitHookSurface::LifecycleHook(LifecycleHookType::PostCommit),
        GitHookSurface::LifecycleHook(LifecycleHookType::FsmonitorWatchman),
    ];

    for hook in hooks {
        log_info(&format!(
            "{} - {} (contract: {})",
            hook.hook_name(),
            hook.description(),
            hook.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_lfs_hooks() -> Result<(), Box<dyn std::error::Error>> {
    log_header("LFS HOOKS");
    println!();

    let hooks = vec![
        GitHookSurface::LfsHook(LfsHookType::PrePush),
        GitHookSurface::LfsHook(LfsHookType::PostCheckout),
        GitHookSurface::LfsHook(LfsHookType::Clean),
        GitHookSurface::LfsHook(LfsHookType::Smudge),
    ];

    for hook in hooks {
        log_info(&format!(
            "{} - {} (contract: {})",
            hook.hook_name(),
            hook.description(),
            hook.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_attribute_filters() -> Result<(), Box<dyn std::error::Error>> {
    log_header("ATTRIBUTE FILTERS");
    println!();

    let filters = vec![
        GitHookSurface::AttributeFilter(AttributeFilterType::Clean),
        GitHookSurface::AttributeFilter(AttributeFilterType::Smudge),
        GitHookSurface::AttributeFilter(AttributeFilterType::Diff),
        GitHookSurface::AttributeFilter(AttributeFilterType::Merge),
        GitHookSurface::AttributeFilter(AttributeFilterType::Text),
        GitHookSurface::AttributeFilter(AttributeFilterType::Binary),
    ];

    for filter in filters {
        log_info(&format!(
            "{} - {} (contract: {})",
            filter.hook_name(),
            filter.description(),
            filter.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_config_hooks() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CONFIG HOOKS");
    println!();

    let hooks = vec![
        GitHookSurface::ConfigHook(ConfigHookType::IncludeIf),
        GitHookSurface::ConfigHook(ConfigHookType::Alias),
        GitHookSurface::ConfigHook(ConfigHookType::Credential),
        GitHookSurface::ConfigHook(ConfigHookType::CorePager),
        GitHookSurface::ConfigHook(ConfigHookType::CoreEditor),
    ];

    for hook in hooks {
        log_info(&format!(
            "{} - {} (contract: {})",
            hook.hook_name(),
            hook.description(),
            hook.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_notes_hooks() -> Result<(), Box<dyn std::error::Error>> {
    log_header("NOTES HOOKS");
    println!();

    let hooks = vec![
        GitHookSurface::NotesHook(NotesHookType::PreNotesEdit),
        GitHookSurface::NotesHook(NotesHookType::PostNotesEdit),
        GitHookSurface::NotesHook(NotesHookType::NotesRewrite),
        GitHookSurface::NotesHook(NotesHookType::NotesRef),
    ];

    for hook in hooks {
        log_info(&format!(
            "{} - {} (contract: {})",
            hook.hook_name(),
            hook.description(),
            hook.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_custom_refs() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CUSTOM REFS");
    println!();

    let refs = vec![
        GitHookSurface::CustomRef(CustomRefType::CustomRef),
        GitHookSurface::CustomRef(CustomRefType::Reflog),
        GitHookSurface::CustomRef(CustomRefType::SymbolicRef),
        GitHookSurface::CustomRef(CustomRefType::Head),
    ];

    for hook in refs {
        log_info(&format!(
            "{} - {} (contract: {})",
            hook.hook_name(),
            hook.description(),
            hook.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_worktree_hooks() -> Result<(), Box<dyn std::error::Error>> {
    log_header("WORKTREE HOOKS");
    println!();

    let hooks = vec![
        GitHookSurface::WorktreeHook(WorktreeHookType::PreWorktreeCreate),
        GitHookSurface::WorktreeHook(WorktreeHookType::PostWorktreeCreate),
        GitHookSurface::WorktreeHook(WorktreeHookType::PreWorktreeRemove),
        GitHookSurface::WorktreeHook(WorktreeHookType::PostWorktreeRemove),
        GitHookSurface::WorktreeHook(WorktreeHookType::WorktreeSwitch),
        GitHookSurface::WorktreeHook(WorktreeHookType::WorktreeSync),
    ];

    for hook in hooks {
        log_info(&format!(
            "{} - {} (contract: {})",
            hook.hook_name(),
            hook.description(),
            hook.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_external_tools() -> Result<(), Box<dyn std::error::Error>> {
    log_header("EXTERNAL TOOLS");
    println!();

    let tools = vec![
        GitHookSurface::ExternalTool(ExternalToolType::Husky),
        GitHookSurface::ExternalTool(ExternalToolType::Lefthook),
        GitHookSurface::ExternalTool(ExternalToolType::PreCommit),
        GitHookSurface::ExternalTool(ExternalToolType::GitHooks),
        GitHookSurface::ExternalTool(ExternalToolType::CustomBinary),
    ];

    for tool in tools {
        log_info(&format!(
            "{} - {} (contract: {})",
            tool.hook_name(),
            tool.description(),
            tool.contract_type()
        ));
    }
    println!();
    Ok(())
}

fn demonstrate_runtime_proxies() -> Result<(), Box<dyn std::error::Error>> {
    log_header("RUNTIME PROXIES");
    println!();

    let proxies = vec![
        GitHookSurface::RuntimeProxy(RuntimeProxyType::HttpProxy),
        GitHookSurface::RuntimeProxy(RuntimeProxyType::UrlRewrite),
        GitHookSurface::RuntimeProxy(RuntimeProxyType::RemoteProxy),
        GitHookSurface::RuntimeProxy(RuntimeProxyType::PagerProxy),
        GitHookSurface::RuntimeProxy(RuntimeProxyType::EditorProxy),
    ];

    for proxy in proxies {
        log_info(&format!(
            "{} - {} (contract: {})",
            proxy.hook_name(),
            proxy.description(),
            proxy.contract_type()
        ));
    }
    println!();
    Ok(())
}
