package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLoadSiteBundleRejectsDuplicateSlugs(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[]}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "about.json"), `{"slug":"","widgets":[]}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected duplicate slug error")
	}
}

func TestLoadSiteBundleRejectsInvalidOutputFolder(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"../bad"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[]}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected invalid output_folder error")
	}
}

func TestLoadSiteBundleWarnsOnUnknownTopLevelKeys(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite","mystery":"x"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[],"unused_flag":true}`)

	_, warnings, err := loadSiteBundle(siteDir)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(warnings) != 2 {
		t.Fatalf("expected 2 warnings, got %d", len(warnings))
	}
}

func TestLoadSiteBundleRejectsMissingWidgetsField(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"kometa","output_folder":"KometaWebsite"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":""}`)

	_, _, err := loadSiteBundle(siteDir)
	if err == nil {
		t.Fatal("expected widgets required error")
	}
}

func createTestSiteDir(t *testing.T) string {
	t.Helper()

	root := t.TempDir()
	siteDir := filepath.Join(root, "sites", "test-site")
	pagesDir := filepath.Join(siteDir, "pages")
	if err := os.MkdirAll(pagesDir, 0o755); err != nil {
		t.Fatalf("cannot create test directories: %v", err)
	}
	return siteDir
}

func writeJSONFile(t *testing.T, path, content string) {
	t.Helper()

	if err := os.WriteFile(path, []byte(content), 0o644); err != nil {
		t.Fatalf("cannot write %s: %v", path, err)
	}
}
