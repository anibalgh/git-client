use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use rmerge_application::ports::out::GitConfigPort;
use rmerge_infrastructure::git::GitConfigAdapter;
use rmerge_infrastructure::ipc::{IpcChannel, IpcMessage};

#[derive(Parser, Debug)]
#[command(
    name = "rmerge",
    author = "AnibalGH",
    version = rmerge_domain::APP_VERSION,
    about = rmerge_domain::APP_DESCRIPTION
)]
struct Cli {
    /// Ruta del repositorio Git a abrir (por defecto: directorio actual)
    #[arg(value_name = "REPO_PATH", default_value = ".")]
    path: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Abre la herramienta de resolución de conflictos 3-Way Merge (compatible con git mergetool)
    Mergetool {
        /// Archivo ancestro común (Base)
        base: PathBuf,
        /// Versión local (Ours)
        local: PathBuf,
        /// Versión remota (Theirs)
        remote: PathBuf,
        /// Archivo de salida para el resultado combinado
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Abre la vista de culpa/historial para un archivo
    Blame {
        file: PathBuf,
        line: Option<u32>,
    },
    /// Realiza una búsqueda avanzada en el historial de commits
    Search {
        query: String,
    },
    /// Verifica o configura la identidad de Git (user.name y user.email)
    ConfigIdentity {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long, default_value_t = false)]
        global: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 1. Manejo de subcomandos especiales
    if let Some(ref cmd) = cli.command {
        match cmd {
            Commands::ConfigIdentity { name, email, global } => {
                let config_adapter = Arc::new(GitConfigAdapter::new());
                let canonical_path = std::fs::canonicalize(&cli.path).unwrap_or(cli.path.clone());

                if let (Some(n), Some(e)) = (name, email) {
                    let author = rmerge_domain::entities::GitAuthor::new(n, e);
                    let scope = if *global {
                        rmerge_domain::entities::ConfigScope::Global
                    } else {
                        rmerge_domain::entities::ConfigScope::Local
                    };
                    config_adapter.set_author(&canonical_path, &author, scope).await?;
                    println!("Identidad configurada exitosamente ({:?}): {} <{}>", scope, author.name, author.email);
                    return Ok(());
                } else {
                    let effective = config_adapter.get_effective_author(&canonical_path).await?;
                    match effective {
                        Some(a) => println!("Autor activo: {} <{}>", a.name, a.email),
                        None => println!("No hay autor de Git configurado para este repositorio."),
                    }
                    return Ok(());
                }
            }
            Commands::Mergetool { base, local, remote, output } => {
                let msg = IpcMessage::OpenMergeTool {
                    base: base.to_string_lossy().to_string(),
                    local: local.to_string_lossy().to_string(),
                    remote: remote.to_string_lossy().to_string(),
                    output: output.to_string_lossy().to_string(),
                };
                if IpcChannel::send_to_active_instance(&msg).await.is_ok() {
                    println!("Abriendo conflicto en la instancia activa de Git-Client...");
                    return Ok(());
                }
            }
            Commands::Blame { file, line } => {
                let msg = IpcMessage::Blame {
                    file: file.to_string_lossy().to_string(),
                    line: *line,
                };
                if IpcChannel::send_to_active_instance(&msg).await.is_ok() {
                    println!("Abriendo blame en la instancia activa de Git-Client...");
                    return Ok(());
                }
            }
            Commands::Search { query } => {
                let msg = IpcMessage::Search { query: query.clone() };
                if IpcChannel::send_to_active_instance(&msg).await.is_ok() {
                    println!("Enviando búsqueda a la instancia activa de Git-Client...");
                    return Ok(());
                }
            }
        }
    }

    // 2. Comando estándar: abrir repositorio
    let target_path = std::fs::canonicalize(&cli.path).unwrap_or_else(|_| cli.path.clone());
    let msg = IpcMessage::OpenRepository {
        path: target_path.to_string_lossy().to_string(),
    };

    // Intentar enviar a instancia activa
    if IpcChannel::send_to_active_instance(&msg).await.is_ok() {
        println!("Repositorio abierto en la instancia existente de Git-Client: {}", target_path.display());
        return Ok(());
    }

    // Si no hay instancia activa, lanzar el ejecutable GUI de rmerge
    println!("Iniciando Git-Client (rmerge-gui) para: {}", target_path.display());
    let mut gui_cmd = std::process::Command::new("rmerge-gui");
    gui_cmd.arg(target_path);

    match gui_cmd.spawn() {
        Ok(_) => Ok(()),
        Err(e) => {
            // Si rmerge-gui no está en el PATH del sistema aún, buscar en el mismo directorio del ejecutable actual
            if let Ok(current_exe) = std::env::current_exe() {
                if let Some(parent) = current_exe.parent() {
                    let exe_name = if cfg!(target_os = "windows") { "rmerge-gui.exe" } else { "rmerge-gui" };
                    let local_gui = parent.join(exe_name);
                    if local_gui.exists() {
                        let _ = std::process::Command::new(local_gui)
                            .arg(&cli.path)
                            .spawn();
                        return Ok(());
                    }
                }
            }
            eprintln!("Aviso: La interfaz gráfica rmerge-gui no pudo iniciarse en segundo plano: {e}");
            Ok(())
        }
    }
}
