package main

import (
	"fmt"
	"path/filepath"
	"sort"
	"strings"
)

type PageRoute struct {
	Slug          string
	SourcePath    string
	OutputRelPath string
	DirRelPath    string
}

type RouteIndex struct {
	Ordered []PageRoute
	BySlug  map[string]PageRoute
}

func buildRouteIndex(bundle SiteBundle) (RouteIndex, error) {
	routes := make([]PageRoute, 0, len(bundle.Pages))
	bySlug := make(map[string]PageRoute, len(bundle.Pages))

	for _, pageFile := range bundle.Pages {
		slug, err := normalizedSlug(pageFile.Page.Slug)
		if err != nil {
			return RouteIndex{}, fmt.Errorf("%s -> %w", pageFile.Path, err)
		}

		route := PageRoute{
			Slug:          slug,
			SourcePath:    pageFile.Path,
			OutputRelPath: outputRelPathForSlug(slug),
			DirRelPath:    outputDirRelPathForSlug(slug),
		}
		routes = append(routes, route)
		bySlug[slug] = route
	}

	sort.Slice(routes, func(i, j int) bool {
		return routes[i].OutputRelPath < routes[j].OutputRelPath
	})

	return RouteIndex{
		Ordered: routes,
		BySlug:  bySlug,
	}, nil
}

func outputRelPathForSlug(slug string) string {
	if slug == "" {
		return "index.html"
	}
	return filepath.Join(filepath.FromSlash(slug), "index.html")
}

func outputDirRelPathForSlug(slug string) string {
	if slug == "" {
		return ""
	}
	return filepath.FromSlash(slug)
}

func normalizedSlug(slug string) (string, error) {
	s := strings.TrimSpace(slug)
	s = strings.Trim(s, "/")
	if s == "" {
		return "", nil
	}
	if strings.Contains(s, "..") {
		return "", fmt.Errorf(`invalid slug %q`, slug)
	}
	if strings.ContainsAny(s, `\`) {
		return "", fmt.Errorf(`invalid slug %q`, slug)
	}
	if strings.Contains(s, "//") {
		return "", fmt.Errorf(`invalid slug %q`, slug)
	}
	return filepath.ToSlash(s), nil
}

func resolveInternalSlugReference(current PageRoute, rawHref string, routesBySlug map[string]PageRoute) (string, error) {
	href := strings.TrimSpace(rawHref)
	if isExternalOrSpecialHref(href) {
		return href, nil
	}

	targetSlug, fragment, err := parseSlugReference(href)
	if err != nil {
		return "", err
	}
	targetRoute, ok := routesBySlug[targetSlug]
	if !ok {
		return "", fmt.Errorf("unknown internal slug reference %q", targetSlug)
	}

	base := relativeDirLink(current.DirRelPath, targetRoute.DirRelPath)
	return base + fragment, nil
}

func parseSlugReference(href string) (string, string, error) {
	frag := ""
	if i := strings.Index(href, "#"); i >= 0 {
		frag = href[i:]
		href = href[:i]
	}
	href = strings.TrimSpace(href)
	if href == "" {
		return "", frag, nil
	}
	if href == "/" {
		return "", frag, nil
	}
	slug, err := normalizedSlug(href)
	if err != nil {
		return "", "", err
	}
	return slug, frag, nil
}

func relativeDirLink(fromDir, toDir string) string {
	from := "."
	if fromDir != "" {
		from = fromDir
	}
	to := "."
	if toDir != "" {
		to = toDir
	}

	rel, err := filepath.Rel(from, to)
	if err != nil {
		return "./"
	}
	rel = filepath.ToSlash(rel)
	if rel == "." {
		return "./"
	}
	if strings.HasSuffix(rel, "/") {
		return rel
	}
	return rel + "/"
}

func assetPrefixForDepth(dirRelPath string) string {
	if dirRelPath == "" {
		return ""
	}
	segments := strings.Split(filepath.ToSlash(dirRelPath), "/")
	return strings.Repeat("../", len(segments))
}

func resolveAssetHrefForPage(raw string, route PageRoute) string {
	href := strings.TrimSpace(raw)
	if href == "" {
		return ""
	}
	if isExternalOrSpecialHref(href) || strings.HasPrefix(href, "/") {
		return href
	}
	href = strings.TrimPrefix(href, "./")
	return assetPrefixForDepth(route.DirRelPath) + href
}

func isExternalOrSpecialHref(href string) bool {
	if strings.HasPrefix(href, "#") {
		return true
	}
	lowered := strings.ToLower(href)
	return strings.HasPrefix(lowered, "http://") ||
		strings.HasPrefix(lowered, "https://") ||
		strings.HasPrefix(lowered, "mailto:") ||
		strings.HasPrefix(lowered, "tel:") ||
		strings.HasPrefix(lowered, "data:") ||
		strings.HasPrefix(lowered, "javascript:")
}
