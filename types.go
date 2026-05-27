package main

import (
	"encoding/json"
	"strings"
)

type SiteBundle struct {
	SiteDir  string
	SitePath string
	Site     SiteConfig
	Pages    []SitePageFile
}

type SitePageFile struct {
	Path       string
	Page       PageConfig
	HasSlug    bool
	HasWidgets bool
}

type ConfigWarning struct {
	FilePath string
	Key      string
}

func (w ConfigWarning) String() string {
	return w.FilePath + " -> unknown key: " + w.Key
}

type SiteConfig struct {
	SiteID         string            `json:"site_id"`
	OutputFolder   string            `json:"output_folder"`
	Theme          map[string]string `json:"theme,omitempty"`
	Typography     TypographyConfig  `json:"typography,omitempty"`
	StoreIcons     StoreIcons        `json:"store_icons,omitempty"`
	SubscribeBlock SubscribeBlock    `json:"subscribe_block,omitempty"`
	Social         SocialSection     `json:"social,omitempty"`
	Header         HeaderConfig      `json:"header,omitempty"`
	Footer         FooterConfig      `json:"footer,omitempty"`
	BaseURL        string            `json:"base_url,omitempty"`
	Widgets        WidgetsConfig     `json:"widgets,omitempty"`
}

type HeaderConfig struct {
	Brand HeaderBrand `json:"brand,omitempty"`
	Nav   []NavItem   `json:"nav,omitempty"`
}

type HeaderBrand struct {
	Logo string `json:"logo,omitempty"`
	Text string `json:"text,omitempty"`
}

type PageConfig struct {
	Slug    string          `json:"slug"`
	Widgets []WidgetNode    `json:"widgets"`
	Title   string          `json:"title,omitempty"`
	SEO     PageSEO         `json:"seo,omitempty"`
	Hero    json.RawMessage `json:"hero,omitempty"`
	Layout  PageLayout      `json:"layout,omitempty"`
}

type PageSEO struct {
	Description  string `json:"description,omitempty"`
	OGImage      string `json:"og_image,omitempty"`
	CanonicalURL string `json:"canonical_url,omitempty"`
}

type PageLayout struct {
	HideHeader bool `json:"hide_header,omitempty"`
	HideFooter bool `json:"hide_footer,omitempty"`
}

type WidgetNode struct {
	Type    string                     `json:"type"`
	ID      string                     `json:"id,omitempty"`
	Enabled *bool                      `json:"enabled,omitempty"`
	Props   map[string]json.RawMessage `json:"props,omitempty"`
}

type TypographyConfig struct {
	GoogleFontsStylesheetHref string `json:"google_fonts_stylesheet_href,omitempty"`
	FontFamilyHeading         string `json:"font_family_heading,omitempty"`
	FontFamilyBody            string `json:"font_family_body,omitempty"`
}

// NavItem is one header link (`site.json` → `header.nav`).
type NavItem struct {
	Label        string `json:"label"`
	Href         string `json:"href"`
	OpenInNewTab bool   `json:"open_in_new_tab"`
}

type StoreIcons map[string]string

func defaultStoreIconPaths() StoreIcons {
	return StoreIcons{
		"google_play": "Images/gp-store-icon.png",
		"app_store":   "Images/appstore-store-icon.png",
		"galaxy":      "Images/galaxy-store-icon.png",
		"amazon":      "Images/amazon-store-icon.png",
	}
}

func (m StoreIcons) withDefaults() StoreIcons {
	out := make(StoreIcons)
	for k, v := range defaultStoreIconPaths() {
		out[k] = v
	}
	if m == nil {
		return out
	}
	for k, v := range m {
		v = strings.TrimSpace(v)
		if v != "" {
			out[k] = v
		}
	}
	return out
}

// SubscribeBlock is shown inside each catalog app card above store buttons when at least one link has a URL.
type SubscribeBlock struct {
	Title string          `json:"title"`
	Links []SubscribeLink `json:"links"`
}

type SubscribeLink struct {
	Label string `json:"label"`
	URL   string `json:"url"`
}

// WidgetsConfig tunes front-end scripts; omitted fields use defaults matching the original hardcoded behavior.
type WidgetsConfig struct {
	ScrollReveal ScrollRevealWidgetConfig `json:"scroll_reveal"`
	Carousel     CarouselWidgetConfig     `json:"carousel"`
	SplitWidget  SplitWidgetConfig        `json:"split_widget"`
}

type ScrollRevealWidgetConfig struct {
	RespectReducedMotion *bool    `json:"respect_reduced_motion,omitempty"`
	RootMargin           string   `json:"root_margin,omitempty"`
	Threshold            *float64 `json:"threshold,omitempty"`
}

type CarouselWidgetConfig struct {
	SwipeThresholdPx   *int  `json:"swipe_threshold_px,omitempty"`
	KeyboardNavigation *bool `json:"keyboard_navigation,omitempty"`
}

type SplitWidgetConfig struct {
	KeyboardNavigation *bool `json:"keyboard_navigation,omitempty"`
}

// SocialSection supports either legacy github_url / linkedin_url / facebook_url or an explicit links array.
type SocialSection struct {
	Links    []SocialLink `json:"links,omitempty"`
	Github   string       `json:"github_url,omitempty"`
	Linkedin string       `json:"linkedin_url,omitempty"`
	Facebook string       `json:"facebook_url,omitempty"`
}

type SocialLink struct {
	URL       string `json:"url"`
	AriaLabel string `json:"aria_label,omitempty"`
	Icon      string `json:"icon,omitempty"`
	IconImage string `json:"icon_image,omitempty"`
}

func (s SocialSection) resolvedSocialLinks() []SocialLink {
	if len(s.Links) > 0 {
		return s.Links
	}
	var out []SocialLink
	if u := strings.TrimSpace(s.Github); u != "" {
		out = append(out, SocialLink{URL: u, Icon: "github"})
	}
	if u := strings.TrimSpace(s.Linkedin); u != "" {
		out = append(out, SocialLink{URL: u, Icon: "linkedin"})
	}
	if u := strings.TrimSpace(s.Facebook); u != "" {
		out = append(out, SocialLink{URL: u, Icon: "facebook"})
	}
	return out
}

// CatalogApp is one entry in the catalog / apps showcase (legacy single-page config shape).
type CatalogApp struct {
	Image          string      `json:"image"`
	HeaderImage    string      `json:"header_image"`
	SwiperImages   []string    `json:"swiper_images"`
	CardBackground string      `json:"card_background"`
	Title          string      `json:"title"`
	Text1          string      `json:"text_1"`
	Text2          string      `json:"text_2"`
	StatLeftLine1  string      `json:"stat_left_line_1"`
	StatLeftLine2  string      `json:"stat_left_line_2"`
	StatRightLine1 string      `json:"stat_right_line_1"`
	StatRightLine2 string      `json:"stat_right_line_2"`
	GooglePlayURL  string      `json:"google_play_url"`
	AppStoreURL    string      `json:"app_store_url"`
	AmazonStoreURL string      `json:"amazon_store_url"`
	GalaxyStoreURL string      `json:"galaxy_store_url"`
	StoreLinks     []StoreLink `json:"store_links,omitempty"`
}

type StoreLink struct {
	URL       string `json:"url"`
	AriaLabel string `json:"aria_label,omitempty"`
	Icon      string `json:"icon,omitempty"`
	IconImage string `json:"icon_image,omitempty"`
}

type Vacancy struct {
	Role             string   `json:"role"`
	Requirements     []string `json:"requirements"`
	Responsibilities []string `json:"responsibilities"`
	Advantages       []string `json:"advantages"`
	ApplyURL         string   `json:"apply_url"`
	ApplyLabel       string   `json:"apply_label"`
}

type FooterConfig struct {
	Enabled      *bool         `json:"enabled,omitempty"`
	SectionTitle string        `json:"section_title"`
	Contact      FooterContact `json:"contact"`
	PrivacyURL   string        `json:"privacy_url"`
	PrivacyLabel string        `json:"privacy_label"`
	TermsURL     string        `json:"terms_url"`
	TermsLabel   string        `json:"terms_label"`
	Copyright    string        `json:"copyright"`
	CookieNotice string        `json:"cookie_notice"`
}

func (f FooterConfig) isFooterEnabled() bool {
	if f.Enabled == nil {
		return true
	}
	return *f.Enabled
}

type FooterContact struct {
	Paragraphs []string `json:"paragraphs"`
	Email      string   `json:"email"`
}

type splitEntry struct {
	Label string
	Body  string
}
