package main

import (
	"bytes"
	"errors"
	"fmt"
	"image"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"
	"os"
	"path/filepath"
	"strings"
)

var pngFileSignature = []byte{0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A}

func readPNGPixelHeight(path string) (int, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return 0, err
	}
	if len(b) < 24 {
		return 0, errors.New("file too small for png")
	}
	if !bytes.Equal(b[:8], pngFileSignature) {
		return 0, errors.New("not a png signature")
	}
	if string(b[12:16]) != "IHDR" {
		return 0, errors.New("png missing IHDR")
	}
	h := int(b[20])<<24 | int(b[21])<<16 | int(b[22])<<8 | int(b[23])
	if h <= 0 || h > 100_000 {
		return 0, errors.New("invalid png height")
	}
	return h, nil
}

const defaultGameStoreIconHeightPx = 52

// gameStoreIconDisplayScale is applied to the measured PNG height (30% smaller → 70% size).
const gameStoreIconDisplayScale = 0.7

func scaledGameStoreIconHeightPx(measured int) int {
	if measured <= 0 {
		return 0
	}
	return int(float64(measured)*gameStoreIconDisplayScale + 0.5)
}

func gameStoreIconHeightCSS(projectRoot string, icons GameStoreIcons) string {
	def := scaledGameStoreIconHeightPx(defaultGameStoreIconHeightPx)
	gp := strings.TrimSpace(icons["google_play"])
	if gp == "" {
		return fmt.Sprintf("%dpx", def)
	}
	abs, _, err := resolveAssetUnderProject(projectRoot, gp)
	if err != nil {
		return fmt.Sprintf("%dpx", def)
	}
	h, err := readPNGPixelHeight(abs)
	if err != nil {
		return fmt.Sprintf("%dpx", def)
	}
	return fmt.Sprintf("%dpx", scaledGameStoreIconHeightPx(h))
}

const gameIconDisplayScale = 0.8

func gameIconImgSizeAttrs(projectRoot, webPath string) string {
	webPath = strings.TrimSpace(webPath)
	if webPath == "" {
		return ""
	}
	abs, _, err := resolveAssetUnderProject(projectRoot, webPath)
	if err != nil {
		return ""
	}
	f, err := os.Open(abs)
	if err != nil {
		return ""
	}
	defer f.Close()
	cfg, _, err := image.DecodeConfig(f)
	if err != nil {
		return ""
	}
	w := int(float64(cfg.Width)*gameIconDisplayScale + 0.5)
	h := int(float64(cfg.Height)*gameIconDisplayScale + 0.5)
	if w < 1 {
		w = 1
	}
	if h < 1 {
		h = 1
	}
	return fmt.Sprintf(` width="%d" height="%d"`, w, h)
}

func copyConfigAssets(projectRoot, siteRoot string, config Config) error {
	paths := collectAssetPaths(config)
	for _, webPath := range paths {
		if err := copyProjectAsset(projectRoot, siteRoot, webPath); err != nil {
			return err
		}
	}
	return nil
}

func collectAssetPaths(config Config) []string {
	seen := map[string]struct{}{}
	var out []string

	add := func(p string) {
		p = strings.TrimSpace(p)
		if p == "" {
			return
		}
		p = strings.TrimPrefix(p, "./")
		key := filepath.ToSlash(p)
		if _, ok := seen[key]; ok {
			return
		}
		seen[key] = struct{}{}
		out = append(out, p)
	}

	if config.Content != nil {
		add(config.Content["site_icon"])
		add(config.Content["header_logo_image"])
		add(config.Content["cover_image"])
	}
	gsi := config.GameStoreIcons.withDefaults()
	for _, path := range gsi {
		add(path)
	}
	for _, link := range config.Social.resolvedSocialLinks() {
		add(link.IconImage)
	}
	for _, p := range config.Photos {
		add(p)
	}
	for _, o := range config.Offers {
		add(o.Image)
	}
	for _, g := range config.Games {
		add(g.Image)
		add(g.HeaderImage)
		for _, p := range g.SwiperImages {
			add(p)
		}
		for _, sl := range g.StoreLinks {
			add(sl.IconImage)
		}
	}
	return out
}

func copyProjectAsset(projectRoot, siteRoot, webPath string) error {
	srcAbs, rel, err := resolveAssetUnderProject(projectRoot, webPath)
	if err != nil {
		return err
	}
	if _, err := os.Stat(srcAbs); err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("asset not found (referenced in config): %s", webPath)
		}
		return fmt.Errorf("asset %s: %w", webPath, err)
	}
	dst := filepath.Join(siteRoot, rel)
	return copyFile(srcAbs, dst)
}

func resolveAssetUnderProject(projectRoot, webPath string) (srcAbs string, rel string, err error) {
	webPath = strings.TrimSpace(webPath)
	if webPath == "" {
		return "", "", errors.New("empty asset path")
	}
	webPath = strings.TrimPrefix(webPath, "./")
	if strings.Contains(filepath.ToSlash(webPath), "..") {
		return "", "", fmt.Errorf("invalid asset path: %q", webPath)
	}

	rel = filepath.FromSlash(webPath)

	absRoot, err := filepath.Abs(filepath.Clean(projectRoot))
	if err != nil {
		return "", "", err
	}
	joined := filepath.Join(absRoot, rel)
	absSrc, err := filepath.Abs(filepath.Clean(joined))
	if err != nil {
		return "", "", err
	}

	relOut, err := filepath.Rel(absRoot, absSrc)
	if err != nil || strings.HasPrefix(relOut, "..") {
		return "", "", fmt.Errorf("asset path escapes project root: %q", webPath)
	}

	return absSrc, relOut, nil
}
