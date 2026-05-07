package main

import "strings"

type Config struct {
	OutputFolder   string             `json:"output_folder"`
	Theme          map[string]string  `json:"theme"`
	Typography     TypographyConfig   `json:"typography"`
	Nav            []NavItem          `json:"nav"`
	Sections       []SectionSpec      `json:"sections"`
	Content        map[string]string  `json:"content"`
	Offers         []OfferItem        `json:"offers"`
	Photos         []string           `json:"photos"`
	GameStoreIcons GameStoreIcons     `json:"game_store_icons"`
	GameSubscribe  GameSubscribeBlock `json:"game_subscribe"`
	Games          []Game             `json:"games"`
	Vacancies      []Vacancy          `json:"vacancies"`
	Social         SocialSection      `json:"social"`
	Footer         FooterConfig       `json:"footer"`
	Widgets        WidgetsConfig      `json:"widgets"`
}

// TypographyConfig optional; empty fields use built-in Google Fonts URL and stacks.
type TypographyConfig struct {
	GoogleFontsStylesheetHref string `json:"google_fonts_stylesheet_href,omitempty"`
	FontFamilyHeading         string `json:"font_family_heading,omitempty"`
	FontFamilyBody            string `json:"font_family_body,omitempty"`
}

// NavItem is one header link when using custom nav (non-empty nav array replaces legacy nav_* content keys).
type NavItem struct {
	Label          string `json:"label"`
	Href           string `json:"href"`
	OpenInNewTab   bool   `json:"open_in_new_tab"`
}

// SectionSpec controls ordering and visibility inside <main>. ID "cover" is special: only shown above <main>
// when it is the first enabled entry in sections; legacy mode (no sections array) keeps cover behavior from content.
type SectionSpec struct {
	ID      string `json:"id"`
	Enabled *bool  `json:"enabled,omitempty"`
}

func (s SectionSpec) isEnabled() bool {
	if s.Enabled == nil {
		return true
	}
	return *s.Enabled
}

// OfferItem is one card in the "We offer" section (any number of items).
type OfferItem struct {
	Image string `json:"image"`
	Title string `json:"title"`
	Text  string `json:"text"`
}

// GameStoreIcons maps preset keys (e.g. google_play, steam) to image paths shared across game cards.
type GameStoreIcons map[string]string

func defaultGameStoreIconPaths() GameStoreIcons {
	return GameStoreIcons{
		"google_play": "Images/gp-store-icon.png",
		"app_store":   "Images/appstore-store-icon.png",
		"galaxy":      "Images/galaxy-store-icon.png",
		"amazon":      "Images/amazon-store-icon.png",
	}
}

func (m GameStoreIcons) withDefaults() GameStoreIcons {
	out := make(GameStoreIcons)
	for k, v := range defaultGameStoreIconPaths() {
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

// GameSubscribeBlock is shown inside each game card above store buttons when at least one link has a URL.
type GameSubscribeBlock struct {
	Title string              `json:"title"`
	Links []GameSubscribeLink `json:"links"`
}

type GameSubscribeLink struct {
	Label string `json:"label"`
	URL   string `json:"url"`
}

// WidgetsConfig tunes front-end scripts; omitted fields use defaults matching the original hardcoded behavior.
type WidgetsConfig struct {
	ScrollReveal ScrollRevealWidgetConfig `json:"scroll_reveal"`
	GameSwiper   GameSwiperWidgetConfig   `json:"game_swiper"`
	SplitWidget  SplitWidgetConfig        `json:"split_widget"`
}

type ScrollRevealWidgetConfig struct {
	RespectReducedMotion *bool    `json:"respect_reduced_motion,omitempty"`
	RootMargin           string   `json:"root_margin,omitempty"`
	Threshold            *float64 `json:"threshold,omitempty"`
}

type GameSwiperWidgetConfig struct {
	SwipeThresholdPx *int `json:"swipe_threshold_px,omitempty"`
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

type Game struct {
	Image          string          `json:"image"`
	HeaderImage    string          `json:"header_image"`
	SwiperImages   []string        `json:"swiper_images"`
	CardBackground string          `json:"card_background"`
	Title          string          `json:"title"`
	Text1          string          `json:"text_1"`
	Text2          string          `json:"text_2"`
	StatLeftLine1  string          `json:"stat_left_line_1"`
	StatLeftLine2  string          `json:"stat_left_line_2"`
	StatRightLine1 string          `json:"stat_right_line_1"`
	StatRightLine2 string          `json:"stat_right_line_2"`
	GooglePlayURL  string          `json:"google_play_url"`
	AppStoreURL    string          `json:"app_store_url"`
	AmazonStoreURL string          `json:"amazon_store_url"`
	GalaxyStoreURL string          `json:"galaxy_store_url"`
	StoreLinks     []GameStoreLink `json:"store_links,omitempty"`
}

type GameStoreLink struct {
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
