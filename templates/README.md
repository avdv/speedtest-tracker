# Template Structure

This directory contains all HTML templates for the Speedtest Tracker web interface.

## Directory Layout

```
templates/
├── base.html              # Base layout template with shared header/nav/styles
├── login.html             # Standalone login page (no header/nav)
├── pages/                 # All page templates
│   ├── admin.html         # Admin dashboard
│   ├── api-tokens.html    # API token management
│   ├── dashboard.html     # Public dashboard with charts
│   ├── edit-token.html    # Edit API token page
│   ├── profile.html       # User profile settings
│   ├── results.html       # Speed test results list
│   └── run-test.html      # Manual speed test execution
└── components/            # Reusable UI components
    └── stat-card.html     # Statistics card widget
```

## Template System

We use **Askama** - a type-safe, compiled Jinja2-like template engine for Rust.

### Base Template

`base.html` provides the master layout with customizable blocks:

- `{% block title %}` - Page title
- `{% block extra_head %}` - Additional `<head>` content (scripts, meta tags)
- `{% block extra_styles %}` - Page-specific CSS
- `{% block header %}` - Override header/navigation
- `{% block content %}` - Main page content (required)
- `{% block extra_scripts %}` - Page-specific JavaScript

### Creating a New Page

1. Create your template in `templates/pages/`
2. Extend the base template:

```html
{% extends "../base.html" %}

{% block title %}My Page{% endblock %}

{% block content %}
<h2>My Content</h2>
<div class="card">
    <p>Page content goes here</p>
</div>
{% endblock %}
```

3. Create a Rust struct in `src/handlers.rs`:

```rust
#[derive(Template)]
#[template(path = "pages/my-page.html")]
pub struct MyPageTemplate {
    // Your data fields
}
```

4. Create a handler function and add route in `src/main.rs`

### Using Components

Include reusable components in your templates:

```html
{% include "components/stat-card.html" with label="Total Tests", value=stats.total %}
```

### Available CSS Classes

Common classes defined in `base.html`:

- `.card` - White content card with shadow
- `.stat-card` - Statistics display card
- `.stat-label` / `.stat-value` / `.stat-unit` - Stat formatting
- `.status-badge` - Status indicator
- `.empty-state` - Empty state messaging
- `.detail-item` / `.detail-label` / `.detail-value` - Detail displays

### Askama Features

Supported template features:

- **Variables**: `{{ variable }}`
- **Conditionals**: `{% if condition %} ... {% endif %}`
- **Loops**: `{% for item in items %} ... {% endfor %}`
- **Filters**: `{{ value|format }}`
- **Rust expressions**: `{{ format!("{:.2}", value) }}`
- **Includes**: `{% include "path/to/component.html" %}`
- **Inheritance**: `{% extends "base.html" %}` and `{% block name %}`

### Styling Guidelines

- Page-specific styles go in `{% block extra_styles %}`
- Keep styles scoped to avoid conflicts
- Use existing base classes when possible
- Follow mobile-first responsive design

## Resources

- [Askama Documentation](https://djc.github.io/askama/)
- [Askama GitHub](https://github.com/djc/askama)
