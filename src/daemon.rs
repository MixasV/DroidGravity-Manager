use anyhow::{Result, Context};
use std::path::PathBuf;

#[cfg(windows)]
use std::process::{Command, Stdio};

pub async fn start_foreground() -> Result<()> {
    println!("Starting proxy server in foreground...");
    
    let config = crate::config::load_config()?;
    
    println!("Port: {}", config.proxy.port);
    println!("API Key: {}", config.proxy.api_key);
    println!();
    println!("Press Ctrl+C to stop");
    
    // Start proxy server
    let proxy_config = crate::proxy::config::ProxyConfig {
        port: config.proxy.port,
        api_key: config.proxy.api_key.clone(),
        allow_lan_access: config.proxy.allow_lan_access,
    };
    crate::proxy::start_server(proxy_config).await?;
    
    Ok(())
}

pub async fn start_background() -> Result<()> {
    // Check if already running
    if is_running().await? {
        anyhow::bail!("Proxy is already running. Use 'drovity stop' first.");
    }
    
    let config = crate::config::load_config()?;
    let pid_file = get_pid_file()?;
    
    // Get current executable path
    let exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    #[cfg(unix)]
    {
        // Unix: fork and daemonize
        use nix::unistd::{fork, ForkResult, setsid};
        use std::os::unix::io::AsRawFd;
        
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // Parent: save PID and exit
                std::fs::write(&pid_file, child.to_string())?;
                println!("[SUCCESS] Proxy started in background (PID: {})", child);
                println!("   Port: {}", config.proxy.port);
                println!("   Use 'drovity stop' to stop the server");
                return Ok(());
            }
            Ok(ForkResult::Child) => {
                // Child: create new session and continue
                setsid().context("Failed to create new session")?;
                
                // Redirect stdout/stderr to log file
                let log_file = crate::config::get_config_dir()?.join("proxy.log");
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_file)?;
                
                let fd = file.as_raw_fd();
                nix::unistd::dup2(fd, std::io::stdout().as_raw_fd())?;
                nix::unistd::dup2(fd, std::io::stderr().as_raw_fd())?;
                
                // Start server
                let proxy_config = crate::proxy::config::ProxyConfig {
                    port: config.proxy.port,
                    api_key: config.proxy.api_key.clone(),
                    allow_lan_access: config.proxy.allow_lan_access,
                };
                crate::proxy::start_server(proxy_config).await?;
            }
            Err(e) => {
                anyhow::bail!("Fork failed: {}", e);
            }
        }
    }
    
    #[cfg(windows)]
    {
        // Windows: spawn detached process
        let child = Command::new(&exe)
            .arg("start")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn background process")?;
        
        let pid = child.id();
        std::fs::write(&pid_file, pid.to_string())?;
        
        println!("[SUCCESS] Proxy started in background (PID: {})", pid);
        println!("   Port: {}", config.proxy.port);
        println!("   Use 'drovity stop' to stop the server");
    }
    
    Ok(())
}

pub async fn stop() -> Result<()> {
    let pid_file = get_pid_file()?;
    
    if !pid_file.exists() {
        anyhow::bail!("Proxy is not running (no PID file found)");
    }
    
    let pid_str = std::fs::read_to_string(&pid_file)
        .context("Failed to read PID file")?;
    let pid: i32 = pid_str.trim().parse()
        .context("Invalid PID in PID file")?;
    
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        
        kill(Pid::from_raw(pid), Signal::SIGTERM)
            .context("Failed to send SIGTERM to process")?;
    }
    
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output()
            .context("Failed to kill process")?;
    }
    
    // Remove PID file
    std::fs::remove_file(&pid_file)?;
    
    println!("[SUCCESS] Proxy stopped (PID: {})", pid);
    
    Ok(())
}

pub async fn status() -> Result<()> {
    let is_running = is_running().await?;
    let config = crate::config::load_config()?;
    
    if is_running {
        let pid_file = get_pid_file()?;
        let pid = std::fs::read_to_string(&pid_file)?;
        
        println!("Status: Running");
        println!("PID: {}", pid.trim());
        println!("Port: {}", config.proxy.port);
        println!("Address: http://127.0.0.1:{}", config.proxy.port);
    } else {
        println!("Status: Stopped");
    }
    
    Ok(())
}

pub async fn is_running() -> Result<bool> {
    let pid_file = get_pid_file()?;
    
    if !pid_file.exists() {
        return Ok(false);
    }
    
    let pid_str = std::fs::read_to_string(&pid_file)?;
    let pid: i32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Invalid PID file, remove it
            let _ = std::fs::remove_file(&pid_file);
            return Ok(false);
        }
    };
    
    // Check if process exists
    #[cfg(unix)]
    {
        use nix::sys::signal::kill;
        use nix::unistd::Pid;
        
        match kill(Pid::from_raw(pid), None) {
            Ok(_) => Ok(true),
            Err(_) => {
                // Process doesn't exist, remove stale PID file
                let _ = std::fs::remove_file(&pid_file);
                Ok(false)
            }
        }
    }
    
    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let exists = output_str.contains(&pid.to_string());
        
        if !exists {
            let _ = std::fs::remove_file(&pid_file);
        }
        
        Ok(exists)
    }
}

fn get_pid_file() -> Result<PathBuf> {
    let config_dir = crate::config::get_config_dir()?;
    Ok(config_dir.join("drovity.pid"))
}
