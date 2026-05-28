package main

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestEnforceStrictWarningsFailsOnUnknownTopLevelKey(t *testing.T) {
	warnings := []ConfigWarning{
		{FilePath: "site.json", Key: "mystery"},
	}
	err := enforceStrictWarnings(warnings)
	if err == nil {
		t.Fatal("expected strict failure for unknown top-level key")
	}
	if !strings.Contains(err.Error(), "unknown key: mystery") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestEnforceStrictWarningsIgnoresLegacyHeroWarning(t *testing.T) {
	warnings := []ConfigWarning{
		contentWarning("home.json", `legacy key "hero" is not supported; use widgets (e.g. intro, cover_banner) for page heroes`),
	}
	if err := enforceStrictWarnings(warnings); err != nil {
		t.Fatalf("legacy hero warning should not fail strict mode: %v", err)
	}
}

func TestLoadSiteBundleWarnsOnUnknownWidgetPropKey(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi","typo":true}}]}`)

	_, warnings, err := loadSiteBundle(siteDir)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(warnings) != 1 {
		t.Fatalf("expected 1 warning, got %d: %v", len(warnings), warnings)
	}
	if !strings.Contains(warnings[0].String(), "unknown props key") {
		t.Fatalf("expected unknown props warning, got %q", warnings[0].String())
	}
}

func TestRunCLIStrictFailsOnUnknownTopLevelKey(t *testing.T) {
	root := t.TempDir()
	templateDir := filepath.Join(root, "Template")
	if err := os.MkdirAll(templateDir, 0o755); err != nil {
		t.Fatal(err)
	}
	copyTestTemplateTree(t, "Template", templateDir)

	siteDir := filepath.Join(root, "content", "strict-test")
	pagesDir := filepath.Join(siteDir, "pages")
	if err := os.MkdirAll(pagesDir, 0o755); err != nil {
		t.Fatal(err)
	}
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"test","output_folder":"Results/Test","mystery":"x"}`)
	writeJSONFile(t, filepath.Join(pagesDir, "home.json"), `{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}`)

	t.Chdir(root)

	var stderr bytes.Buffer
	code := runCLI([]string{"--validate", "--strict", "--site", "content/strict-test"}, bytes.NewReader(nil), io.Discard, &stderr)
	if code == 0 {
		t.Fatalf("expected non-zero exit with --strict, stderr=%q", stderr.String())
	}
	if !strings.Contains(stderr.String(), "unknown key: mystery") {
		t.Fatalf("expected unknown key error, got stderr=%q", stderr.String())
	}
}

func TestRunCLIStrictWorksWithValidateOnly(t *testing.T) {
	root := t.TempDir()
	templateDir := filepath.Join(root, "Template")
	if err := os.MkdirAll(templateDir, 0o755); err != nil {
		t.Fatal(err)
	}
	copyTestTemplateTree(t, "Template", templateDir)

	siteDir := filepath.Join(root, "content", "clean")
	pagesDir := filepath.Join(siteDir, "pages")
	if err := os.MkdirAll(pagesDir, 0o755); err != nil {
		t.Fatal(err)
	}
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"test","output_folder":"Results/Test"}`)
	writeJSONFile(t, filepath.Join(pagesDir, "home.json"), `{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}`)

	t.Chdir(root)

	var stdout, stderr bytes.Buffer
	code := runCLI([]string{"--validate", "--strict", "--site", "content/clean"}, bytes.NewReader(nil), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Validation passed:") {
		t.Fatalf("expected validation success, got stdout=%q", stdout.String())
	}
}