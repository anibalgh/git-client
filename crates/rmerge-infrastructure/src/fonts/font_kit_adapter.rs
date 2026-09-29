use font_kit::source::SystemSource;
use rmerge_application::ports::out::FontDiscoveryPort;
use rmerge_domain::errors::DomainError;

pub struct FontKitAdapter;

impl FontKitAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn load_font_data(family_name: &str) -> Option<Vec<u8>> {
        let source = SystemSource::new();
        if let Ok(handle) = source.select_family_by_name(family_name) {
            for f in handle.fonts() {
                if let Ok(font) = f.load() {
                    if let Some(data) = font.copy_font_data() {
                        return Some((*data).clone());
                    }
                }
            }
        }
        None
    }
    pub fn load_font_family_bytes(family_spec: &str) -> Option<Vec<u8>> {
        for single in family_spec.split(',').map(|s| s.trim()) {
            if single.eq_ignore_ascii_case("sans-serif") || single.eq_ignore_ascii_case("monospace") {
                continue;
            }
            if let Some(bytes) = Self::load_font_data(single) {
                return Some(bytes);
            }
        }
        None
    }
}

impl Default for FontKitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FontDiscoveryPort for FontKitAdapter {
    async fn list_system_fonts(&self) -> Result<Vec<String>, DomainError> {
        let source = SystemSource::new();
        let mut families = source
            .all_families()
            .unwrap_or_default();

        // Asegurar familias estándar multiplataforma si la consulta devuelve lista vacía o reducida
        let platform_fallbacks = [
            "DejaVu Sans", "Liberation Sans", "Ubuntu", "Cantarell", "Noto Sans",
            "Segoe UI", "Arial", "Calibri", "Tahoma", "Verdana",
            "SF Pro Text", "Helvetica Neue", "Helvetica", "Inter", "Roboto",
        ];
        for fb in platform_fallbacks {
            if !families.iter().any(|f| f.eq_ignore_ascii_case(fb)) {
                families.push(fb.to_string());
            }
        }

        families.sort();
        families.dedup();
        Ok(families)
    }

    async fn list_monospace_fonts(&self) -> Result<Vec<String>, DomainError> {
        let source = SystemSource::new();
        let all_families = source
            .all_families()
            .unwrap_or_default();

        let mut mono_families = Vec::new();
        for family in all_families {
            let lower = family.to_lowercase();
            let likely_mono = lower.contains("mono")
                || lower.contains("code")
                || lower.contains("console")
                || lower.contains("terminal")
                || lower.contains("courier")
                || lower.contains("fixed")
                || lower.contains("typewriter");

            if likely_mono {
                mono_families.push(family);
            } else if let Ok(handle) = source.select_family_by_name(&family) {
                let is_mono = handle.fonts().iter().any(|f| {
                    f.load().map_or(false, |loaded| loaded.is_monospace())
                });
                if is_mono {
                    mono_families.push(family);
                }
            }
        }

        // Agregar fuentes comunes de desarrollo multiplataforma (Linux, Windows, macOS)
        let dev_mono_fallbacks = [
            "JetBrains Mono", "Fira Code", "Cascadia Code", "Consolas",
            "DejaVu Sans Mono", "Liberation Mono", "Ubuntu Mono", "Noto Sans Mono",
            "Menlo", "Monaco", "SF Mono", "Courier New", "monospace",
        ];
        for fb in dev_mono_fallbacks {
            if !mono_families.iter().any(|f| f.eq_ignore_ascii_case(fb)) {
                mono_families.push(fb.to_string());
            }
        }

        mono_families.sort();
        mono_families.dedup();
        Ok(mono_families)
    }

    async fn load_font_bytes(&self, family_name: &str) -> Result<Option<Vec<u8>>, DomainError> {
        Ok(Self::load_font_family_bytes(family_name))
    }
}
