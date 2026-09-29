use rmerge_domain::entities::ConfigScope;
use crate::state::IdentityModalState;

pub struct IdentityModalView;

impl IdentityModalView {
    /// Genera la representación visual y controles del modal de identidad
    pub fn render_ascii(state: &IdentityModalState) -> String {
        let mut out = String::new();
        out.push_str("╔════════════════════════════════════════════════════════════════╗\n");
        out.push_str("║          CONFIGURAR IDENTIDAD DE AUTOR DE GIT                  ║\n");
        out.push_str("╟────────────────────────────────────────────────────────────────╢\n");
        out.push_str("║ No se detectó user.name o user.email para firmar los commits.  ║\n");
        out.push_str("║ Ingrese sus datos para continuar:                              ║\n");
        out.push_str("║                                                                ║\n");
        out.push_str(&format!("║  Nombre Completo (user.name):  [{:<31}] ║\n", state.name));
        out.push_str(&format!("║  Correo (user.email):          [{:<31}] ║\n", state.email));
        out.push_str("║                                                                ║\n");
        out.push_str("║  Ámbito de Guardado:                                           ║\n");
        match state.scope {
            ConfigScope::Local => {
                out.push_str("║    (*) Solo para este repositorio (.git/config)                ║\n");
                out.push_str("║    ( ) Global para todos mis repositorios (~/.gitconfig)       ║\n");
            }
            ConfigScope::Global => {
                out.push_str("║    ( ) Solo para este repositorio (.git/config)                ║\n");
                out.push_str("║    (*) Global para todos mis repositorios (~/.gitconfig)       ║\n");
            }
        }
        if let Some(ref err) = state.error_message {
            out.push_str(&format!("║  [ERROR] {:<53} ║\n", err));
        }
        out.push_str("║                                                                ║\n");
        out.push_str("║    [ GUARDAR Y CONTINUAR ]          [ CANCELAR ]               ║\n");
        out.push_str("╚════════════════════════════════════════════════════════════════╝\n");
        out
    }
}
