use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SiteBundle {
    pub site_dir: String,
    pub site_path: String,
    pub site: SiteConfig,
    pub pages: Vec<SitePageFile>,
}

#[derive(Debug, Clone)]
pub struct SitePageFile {
    pub path: String,
    pub page: PageConfig,
    pub has_slug: bool,
    pub has_widgets: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    pub file_path: String,
    pub key: String,
    pub detail: String,
}

impl ConfigWarning {
    pub fn content(file_path: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            key: String::new(),
            detail: detail.into(),
        }
    }

    pub fn to_string(&self) -> String {
        if !self.detail.is_empty() {
            format!("{} -> {}", self.file_path, self.detail)
        } else {
            format!("{} -> unknown key: {}", self.file_path, self.key)
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SiteConfig {
    pub site_id: String,
    pub output_folder: String,
    #[serde(default)]
    pub theme: HashMap<String, String>,
    #[serde(default)]
    pub typography: TypographyConfig,
    #[serde(default)]
    pub store_icons: HashMap<String, String>,
    #[serde(default)]
    pub subscribe_block: SubscribeBlock,
    #[serde(default)]
    pub social: SocialSection,
    #[serde(default)]
    pub header: HeaderConfig,
    #[serde(default)]
    pub footer: FooterConfig,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub widgets: WidgetsConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HeaderConfig {
    #[serde(default)]
    pub brand: HeaderBrand,
    #[serde(default)]
    pub nav: Vec<NavItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HeaderBrand {
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageConfig {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub widgets: Vec<WidgetNode>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub seo: PageSeo,
    #[serde(default)]
    pub layout: PageLayout,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageSeo {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub og_image: String,
    #[serde(default)]
    pub canonical_url: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PageLayout {
    #[serde(default)]
    pub hide_header: bool,
    #[serde(default)]
    pub hide_footer: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetNode {
    #[serde(rename = "type")]
    pub widget_type: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub props: HashMap<String, JsonValue>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TypographyConfig {
    #[serde(default)]
    pub google_fonts_stylesheet_href: String,
    #[serde(default)]
    pub font_family_heading: String,
    #[serde(default)]
    pub font_family_body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub open_in_new_tab: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SubscribeBlock {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub links: Vec<SubscribeLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeLink {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct WidgetsConfig {
    #[serde(default)]
    pub scroll_reveal: ScrollRevealWidgetConfig,
    #[serde(default)]
    pub carousel: CarouselWidgetConfig,
    #[serde(default)]
    pub split_widget: SplitWidgetConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScrollRevealWidgetConfig {
    pub respect_reduced_motion: Option<bool>,
    #[serde(default)]
    pub root_margin: String,
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CarouselWidgetConfig {
    pub swipe_threshold_px: Option<i64>,
    pub keyboard_navigation: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SplitWidgetConfig {
    pub keyboard_navigation: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SocialSection {
    #[serde(default)]
    pub links: Vec<SocialLink>,
    #[serde(default)]
    pub github_url: String,
    #[serde(default)]
    pub linkedin_url: String,
    #[serde(default)]
    pub facebook_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SocialLink {
    pub url: String,
    #[serde(default)]
    pub aria_label: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub icon_image: String,
}

impl SocialSection {
    pub fn resolved_links(&self) -> Vec<SocialLink> {
        if !self.links.is_empty() {
            return self.links.clone();
        }
        let mut out = Vec::new();
        for (url, icon) in [
            (&self.github_url, "github"),
            (&self.linkedin_url, "linkedin"),
            (&self.facebook_url, "facebook"),
        ] {
            let u = url.trim();
            if !u.is_empty() {
                out.push(SocialLink {
                    url: u.to_string(),
                    icon: icon.to_string(),
                    ..Default::default()
                });
            }
        }
        out
    }
}

impl Default for SocialLink {
    fn default() -> Self {
        Self {
            url: String::new(),
            aria_label: String::new(),
            icon: String::new(),
            icon_image: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CatalogApp {
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub header_image: String,
    #[serde(default)]
    pub swiper_images: Vec<String>,
    #[serde(default)]
    pub card_background: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text_1: String,
    #[serde(default)]
    pub text_2: String,
    #[serde(default)]
    pub stat_left_line_1: String,
    #[serde(default)]
    pub stat_left_line_2: String,
    #[serde(default)]
    pub stat_right_line_1: String,
    #[serde(default)]
    pub stat_right_line_2: String,
    #[serde(default)]
    pub google_play_url: String,
    #[serde(default)]
    pub app_store_url: String,
    #[serde(default)]
    pub amazon_store_url: String,
    #[serde(default)]
    pub galaxy_store_url: String,
    #[serde(default)]
    pub store_links: Vec<StoreLink>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StoreLink {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub aria_label: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub icon_image: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Vacancy {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub responsibilities: Vec<String>,
    #[serde(default)]
    pub advantages: Vec<String>,
    #[serde(default)]
    pub apply_url: String,
    #[serde(default)]
    pub apply_label: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FooterConfig {
    pub enabled: Option<bool>,
    #[serde(default)]
    pub section_title: String,
    #[serde(default)]
    pub contact: FooterContact,
    #[serde(default)]
    pub privacy_url: String,
    #[serde(default)]
    pub privacy_label: String,
    #[serde(default)]
    pub terms_url: String,
    #[serde(default)]
    pub terms_label: String,
    #[serde(default)]
    pub copyright: String,
    #[serde(default)]
    pub cookie_notice: String,
}

impl FooterConfig {
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FooterContact {
    #[serde(default)]
    pub paragraphs: Vec<String>,
    #[serde(default)]
    pub email: String,
}

pub fn default_store_icons() -> HashMap<String, String> {
    HashMap::from([
        ("google_play".into(), "Images/gp-store-icon.png".into()),
        ("app_store".into(), "Images/appstore-store-icon.png".into()),
        ("galaxy".into(), "Images/galaxy-store-icon.png".into()),
        ("amazon".into(), "Images/amazon-store-icon.png".into()),
    ])
}

impl SiteConfig {
    pub fn store_icons_with_defaults(&self) -> HashMap<String, String> {
        let mut out = default_store_icons();
        for (k, v) in &self.store_icons {
            let v = v.trim();
            if !v.is_empty() {
                out.insert(k.clone(), v.to_string());
            }
        }
        out
    }
}
