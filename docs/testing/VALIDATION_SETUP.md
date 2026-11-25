# Validation dependencies

Use the locked web dependencies and isolated Python validation environment. From the repository root in PowerShell:

The recorded run used Python 3.14.7, Node 24.20.0 and npm 12.0.2.

```powershell
npm --prefix web ci
python -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r requirements-validation.txt
```

`web/package-lock.json` pins Playwright. The browser smoke uses that project-local package. Playwright reuses its browser cache when it has the required Chromium revision; otherwise install the pinned Chromium build with:

```powershell
if (-not $env:PLAYWRIGHT_BROWSERS_PATH) {
    $env:PLAYWRIGHT_BROWSERS_PATH = Join-Path $PWD 'web/.cache/ms-playwright'
}
npm --prefix web exec -- playwright install chromium
```

Keep that PowerShell session open through the smoke run so the browser cache path remains set.

Run the focused timestamp contract test, documentation validation, and desktop/mobile browser smoke:

```powershell
.\.venv\Scripts\python.exe -m unittest discover -s tests -p test_doc_formats.py
.\.venv\Scripts\python.exe scripts/validate_docs.py
npm --prefix web run test:browser
```
