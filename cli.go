package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"io"
	"path/filepath"
	"strconv"
	"strings"
)

func validateSiteBundleOnly(bundle SiteBundle, templateDir string) ([]ConfigWarning, error) {
	if err := checkReferencedSiteAssets(bundle); err != nil {
		return nil, err
	}
	return validateRenderSiteBundle(bundle, templateDir)
}

func printConfigWarnings(stderr io.Writer, warnings []ConfigWarning) {
	for _, warning := range warnings {
		fmt.Fprintln(stderr, "Warning:", warning.String())
	}
}

func buildSiteBundle(projectRoot, siteInput string) (SiteBundle, []ConfigWarning, error) {
	siteDir := resolveSiteDir(projectRoot, siteInput)
	return loadSiteBundle(siteDir)
}

func generateSite(projectRoot, siteInput string, stdout, stderr io.Writer) (string, error) {
	templateDir := filepath.Join(projectRoot, "Template")

	bundle, warnings, err := buildSiteBundle(projectRoot, siteInput)
	if err != nil {
		return "", err
	}
	printConfigWarnings(stderr, warnings)

	outputFolder, err := validatedOutputFolderFor(bundle.Site.OutputFolder, bundle.SitePath)
	if err != nil {
		return "", err
	}

	targetDir := filepath.Join(projectRoot, filepath.FromSlash(outputFolder))
	if err := prepareDestination(targetDir); err != nil {
		return "", err
	}

	if err := copyTemplateStaticAssets(templateDir, targetDir); err != nil {
		return "", err
	}
	if err := copyReferencedSiteAssets(bundle, targetDir); err != nil {
		return "", err
	}

	renderWarnings, err := renderSiteBundle(bundle, targetDir, templateDir)
	if err != nil {
		return "", err
	}
	printConfigWarnings(stderr, renderWarnings)

	fmt.Fprintln(stdout, "Website generated successfully.")
	fmt.Fprintln(stdout, "Output:", targetDir)
	return targetDir, nil
}

func validateSite(projectRoot, siteInput, templateDir string, stdout, stderr io.Writer) error {
	bundle, warnings, err := buildSiteBundle(projectRoot, siteInput)
	if err != nil {
		return err
	}
	printConfigWarnings(stderr, warnings)

	renderWarnings, err := validateSiteBundleOnly(bundle, templateDir)
	if err != nil {
		return err
	}
	printConfigWarnings(stderr, renderWarnings)

	fmt.Fprintln(stdout, "Validation passed:", bundle.SiteDir)
	return nil
}

func listContentBundles(projectRoot string, stdout io.Writer) error {
	bundles, err := discoverContentBundles(projectRoot)
	if err != nil {
		return err
	}
	for _, rel := range bundles {
		fmt.Fprintln(stdout, rel)
	}
	return nil
}

func printBundleList(stdout io.Writer, bundles []string) {
	fmt.Fprintln(stdout, "Available content bundles:")
	for i, rel := range bundles {
		fmt.Fprintf(stdout, "  %d. %s\n", i+1, rel)
	}
}

func resolveInteractiveSiteInput(projectRoot string, reader lineReader, stdout, stderr io.Writer) (string, error) {
	bundles, err := discoverContentBundles(projectRoot)
	if err != nil {
		return "", err
	}

	for {
		fmt.Fprint(stdout, "Enter content bundle directory path (absolute or relative; empty = default or pick from list; ? = list): ")
		siteInput, err := reader.readOptionalLine()
		if err != nil {
			return "", err
		}

		switch {
		case siteInput == "?":
			printBundleList(stdout, bundles)
			continue
		case siteInput == "" && len(bundles) > 1:
			printBundleList(stdout, bundles)
			fmt.Fprint(stdout, "Enter number or path: ")
			choice, err := reader.readOptionalLine()
			if err != nil {
				return "", err
			}
			if resolved, ok := resolveBundleChoice(choice, bundles); ok {
				return resolved, nil
			}
			return choice, nil
		default:
			return siteInput, nil
		}
	}
}

func resolveBundleChoice(choice string, bundles []string) (string, bool) {
	choice = strings.TrimSpace(choice)
	if choice == "" {
		return "", false
	}
	if n, err := strconv.Atoi(choice); err == nil && n >= 1 && n <= len(bundles) {
		return bundles[n-1], true
	}
	return "", false
}

type lineReader interface {
	readOptionalLine() (string, error)
}

type bufioLineReader struct {
	reader interface {
		ReadString(byte) (string, error)
	}
}

func (r bufioLineReader) readOptionalLine() (string, error) {
	return readOptionalLineFrom(r.reader)
}

func readOptionalLineFrom(reader interface {
	ReadString(byte) (string, error)
}) (string, error) {
	line, err := reader.ReadString('\n')
	if err != nil && !errors.Is(err, io.EOF) {
		return "", err
	}
	return strings.TrimSpace(line), nil
}

func runCLI(args []string, stdin io.Reader, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("portfoliowebsitebuilder", flag.ContinueOnError)
	fs.SetOutput(stderr)

	var validate bool
	var listSites bool
	var serve bool
	var siteFlag string
	var servePort int
	fs.BoolVar(&validate, "validate", false, "validate content bundle without writing output")
	fs.BoolVar(&listSites, "list-sites", false, "list content bundles under content/ and exit")
	fs.BoolVar(&serve, "serve", false, "after build, serve the output directory over HTTP on localhost")
	fs.IntVar(&servePort, "port", defaultServePort, "port for --serve (default 8080)")
	fs.StringVar(&siteFlag, "site", "", "content bundle path (relative to project root or absolute); skips interactive prompt")

	if err := fs.Parse(args); err != nil {
		return 2
	}
	if fs.NArg() > 0 {
		fmt.Fprintf(stderr, "Error: unexpected arguments: %s\n", fs.Args())
		return 2
	}

	projectRoot, err := resolveProjectRoot()
	if err != nil {
		fmt.Fprintln(stderr, "Error:", err)
		return 1
	}

	if listSites {
		if err := listContentBundles(projectRoot, stdout); err != nil {
			fmt.Fprintln(stderr, "Error:", err)
			return 1
		}
		return 0
	}

	templateDir := filepath.Join(projectRoot, "Template")
	siteInput := siteFlag

	if siteInput == "" && !validate {
		reader := bufioLineReader{reader: bufio.NewReader(stdin)}
		siteInput, err = resolveInteractiveSiteInput(projectRoot, reader, stdout, stderr)
		if err != nil {
			fmt.Fprintln(stderr, "Error:", err)
			return 1
		}
	}

	if validate {
		if serve {
			fmt.Fprintln(stderr, "Error: cannot use --serve with --validate")
			return 2
		}
		if err := validateSite(projectRoot, siteInput, templateDir, stdout, stderr); err != nil {
			fmt.Fprintln(stderr, "Error:", err)
			return 1
		}
		return 0
	}

	targetDir, err := generateSite(projectRoot, siteInput, stdout, stderr)
	if err != nil {
		fmt.Fprintln(stderr, "Error:", err)
		return 1
	}

	if serve {
		if servePort < 1 || servePort > 65535 {
			fmt.Fprintf(stderr, "Error: invalid --port %d\n", servePort)
			return 2
		}
		if err := serveStaticDir(targetDir, servePort, stdout); err != nil {
			fmt.Fprintln(stderr, "Error:", err)
			return 1
		}
	}
	return 0
}
