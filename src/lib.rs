use wasm_bindgen::prelude::*;
use web_sys::{
    window, Document, HtmlElement, HtmlTextAreaElement, HtmlSelectElement, HtmlInputElement, 
    Element, Headers, Blob, Url, console, RequestInit, Response, Node
};
use js_sys::{Array, JsString, Uint8Array, Reflect, JSON};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileList;
use serde_json;

// Structure to store generated content
struct GeneratedContent {
    latex: String,
    pdf_blob: Option<Blob>,
    pdf_url: Option<String>,
    chat_history: Vec<(String, String)>,
    pdf_size: String,
    template: String,
    ai_provider: String,
}

// Structure to store API keys
struct ApiKeys {
    claude: String,
    perplexity: String,
    mistral: String,
}

// Initialize console error panic hook for better debugging
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Helper function to get document
fn get_document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

// Helper function to create an element with a class
fn create_element_with_class(tag: &str, class_name: &str) -> HtmlElement {
    let document = get_document();
    let element = document.create_element(tag).unwrap();
    element.set_class_name(class_name);
    element.dyn_into::<HtmlElement>().unwrap()
}

// Alert function
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// Main entry point
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    init_panic_hook();
    
    let _window = window().unwrap();
    let document = get_document();
    let body = document.body().expect("document should have a body");

    // Add theme class to body - default to dark mode
    body.set_class_name("dark-theme");
    
    // Create the UI container
    let container = create_element_with_class("div", "container");
    container.set_attribute("style", "width: 100vw; height: 100vh; margin: 0; padding: 0; overflow: hidden")?;
    
    // Header with logo and toggle buttons
    let header = create_element_with_class("div", "app-header");
    
    // Logo and history toggle button
    let logo_container = create_element_with_class("div", "logo-container");
    
    let logo_img = create_element_with_class("img", "logo-img");
    logo_img.set_attribute("src", "logotex.png")?;
    logo_img.set_attribute("alt", "LaTeX AI Logo")?;
    
    logo_container.append_child(&logo_img)?;

    // History toggle button (moved to left)
    let history_toggle = create_element_with_class("button", "header-btn");
    history_toggle.set_id("history-toggle");
    history_toggle.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 19V5M5 12l7-7 7 7"/></svg>"#);
    
    logo_container.append_child(&history_toggle)?;
    header.append_child(&logo_container)?;
    
    // Right side buttons
    let header_buttons = create_element_with_class("div", "header-buttons");
    
    // Profile button
    let profile_toggle = create_element_with_class("button", "header-btn");
    profile_toggle.set_id("profile-toggle");
    profile_toggle.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#);
    
    // Theme toggle with dropdown
    let theme_container = create_element_with_class("div", "theme-container");
    theme_container.set_attribute("style", "position: relative;");
    
    let theme_toggle = create_element_with_class("button", "header-btn");
    theme_toggle.set_id("theme-toggle");
    theme_toggle.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>"#);
    
    let theme_dropdown = create_element_with_class("div", "theme-dropdown");
    theme_dropdown.set_attribute("style", "position: absolute; top: 100%; right: 0; background: hsl(var(--card)); border: 1px solid hsl(var(--border)); border-radius: 0.5rem; padding: 0.5rem; z-index: 100; display: none;");
    
    let light_option = create_element_with_class("button", "theme-option");
    light_option.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg> Light"#);
    
    let dark_option = create_element_with_class("button", "theme-option");
    dark_option.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg> Dark"#);
    
    let system_option = create_element_with_class("button", "theme-option");
    system_option.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><path d="M8 21h8M12 17v4"/></svg> System"#);
    
    theme_dropdown.append_child(&light_option)?;
    theme_dropdown.append_child(&dark_option)?;
    theme_dropdown.append_child(&system_option)?;
    
    theme_container.append_child(&theme_toggle)?;
    theme_container.append_child(&theme_dropdown)?;
    
    header_buttons.append_child(&profile_toggle)?;
    header_buttons.append_child(&theme_container)?;
    header.append_child(&header_buttons)?;
    
    // Main content area - now uses full width
    let main = create_element_with_class("div", "main-content");
    main.set_attribute("style", "width: 100%; height: calc(100vh - 60px); display: grid; grid-template-columns: 280px 1fr; gap: 0; min-height: 0; overflow: hidden; position: relative;")?;
    
    // Left sidebar - History panel
    let history_panel = create_element_with_class("div", "history-panel");
    history_panel.set_id("history-panel");
    
    let history_header = create_element_with_class("div", "history-header");
    history_header.set_inner_html("<h3>Chat History</h3>");
    
    let new_chat_btn = create_element_with_class("button", "btn-primary");
    new_chat_btn.set_id("new-chat-btn");
    new_chat_btn.set_text_content(Some("New Chat"));
    
    let history_list = create_element_with_class("div", "history-list");
    history_list.set_id("history-list");
    
    // Load saved history
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        if let Ok(Some(history)) = storage.get_item("chat_history") {
            history_list.set_inner_html(&history);
        } else {
            history_list.set_inner_html(r#"
                <div class="empty-state">
                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1-2-2V5a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 1-2-2z"></path>
                    </svg>
                    <p>No chat history yet</p>
                </div>
            "#);
        }
    }
    
    history_panel.append_child(&history_header)?;
    history_panel.append_child(&new_chat_btn)?;
    history_panel.append_child(&history_list)?;
    
    // Profile panel
    let profile_panel = create_element_with_class("div", "profile-panel");
    profile_panel.set_id("profile-panel");
    
    let profile_header = create_element_with_class("div", "profile-header");
    profile_header.set_inner_html("<h3>Profile Settings</h3>");
    
    let api_keys_form = create_element_with_class("div", "api-keys-form");
    
    // Claude API Key
    let claude_group = create_element_with_class("div", "form-group");
    let claude_label = create_element_with_class("label", "form-label");
    claude_label.set_text_content(Some("Claude API Key"));
    
    let claude_input = document.create_element("input")?;
    claude_input.set_class_name("form-input");
    claude_input.set_id("claude-key");
    claude_input.set_attribute("type", "password")?;
    claude_input.set_attribute("placeholder", "sk-ant-...")?;
    
    claude_group.append_child(&claude_label)?;
    claude_group.append_child(&claude_input)?;
    
    // Perplexity API Key
    let perplexity_group = create_element_with_class("div", "form-group");
    let perplexity_label = create_element_with_class("label", "form-label");
    perplexity_label.set_text_content(Some("Perplexity API Key"));
    
    let perplexity_input = document.create_element("input")?;
    perplexity_input.set_class_name("form-input");
    perplexity_input.set_id("perplexity-key");
    perplexity_input.set_attribute("type", "password")?;
    perplexity_input.set_attribute("placeholder", "pplx-...")?;
    
    perplexity_group.append_child(&perplexity_label)?;
    perplexity_group.append_child(&perplexity_input)?;
    
    // Mistral API Key
    let mistral_group = create_element_with_class("div", "form-group");
    let mistral_label = create_element_with_class("label", "form-label");
    mistral_label.set_text_content(Some("Mistral API Key"));
    
    let mistral_input = document.create_element("input")?;
    mistral_input.set_class_name("form-input");
    mistral_input.set_id("mistral-key");
    mistral_input.set_attribute("type", "password")?;
    mistral_input.set_attribute("placeholder", "sk-...")?;
    
    mistral_group.append_child(&mistral_label)?;
    mistral_group.append_child(&mistral_input)?;
    
    // Save button
    let save_btn = create_element_with_class("button", "btn-primary");
    save_btn.set_id("save-keys-btn");
    save_btn.set_text_content(Some("Save API Keys"));
    
    api_keys_form.append_child(&claude_group)?;
    api_keys_form.append_child(&perplexity_group)?;
    api_keys_form.append_child(&mistral_group)?;
    api_keys_form.append_child(&save_btn)?;
    
    profile_panel.append_child(&profile_header)?;
    profile_panel.append_child(&api_keys_form)?;
    
    // Left panel - Chat and Settings
    let left_panel = create_element_with_class("div", "left-panel");
    left_panel.set_attribute("style", "overflow: hidden; display: flex; flex-direction: column; background-color: hsl(var(--card)); border-right: 1px solid hsl(var(--border)); height: 100%; transition: transform 0.3s ease;")?;
    
    // Chat history
    let chat_history = create_element_with_class("div", "chat-history");
    chat_history.set_id("chat-history");
    chat_history.set_inner_html(r#"
        <div class="empty-state">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1-2-2V5a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 1-2-2z"></path>
            </svg>
            <p style="font-size: 1rem; max-width: 80%; text-align: center; color: hsl(var(--muted-foreground));">Your chat history will appear here</p>
        </div>
    "#);
    
    // Chat input container
    let chat_input_container = create_element_with_class("div", "chat-input-container");
    
    // Chat textarea
    let chat_textarea = document.create_element("textarea")?;
    chat_textarea.set_class_name("chat-textarea");
    chat_textarea.set_id("chat-input");
    chat_textarea.set_attribute("placeholder", "Describe document you want to generate...")?;
    chat_textarea.set_attribute("rows", "4")?;
    let chat_textarea = chat_textarea.dyn_into::<HtmlTextAreaElement>()?;
    
    // Chat controls
    let chat_controls = create_element_with_class("div", "chat-controls");
    
    // Attachment and more options
    let attachment_container = create_element_with_class("div", "attachment-container");
    
    // Attach button
    let attach_btn = create_element_with_class("button", "attach-btn");
    attach_btn.set_inner_html(r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="17 8 12 3 7 8"></polyline>
            <line x1="12" y1="3" x2="12" y2="15"></line>
        </svg>
    "#);
    
    let file_input = document.create_element("input")?;
    file_input.set_id("file-upload");
    file_input.set_attribute("type", "file")?;
    file_input.set_attribute("accept", ".tex")?;
    file_input.set_attribute("style", "display: none")?;
    
    // More options button
    let more_options_btn = create_element_with_class("button", "more-options-btn");
    more_options_btn.set_inner_html(r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="1"></circle>
            <circle cx="12" cy="5" r="1"></circle>
            <circle cx="12" cy="19" r="1"></circle>
        </svg>
    "#);
    
    // More options dropdown
    let more_options_dropdown = create_element_with_class("div", "more-options-dropdown");
    more_options_dropdown.set_attribute("style", "display: none;");
    
    // Options row container
    let options_row = create_element_with_class("div", "options-row");
    
    // Template Selection
    let template_group = create_element_with_class("div", "form-group");
    let template_label = create_element_with_class("label", "form-label");
    template_label.set_text_content(Some("Template"));
    
    let template_select = document.create_element("select")?;
    template_select.set_class_name("form-select");
    template_select.set_id("template-select");
    
    let templates = ["Article", "Report", "IEEEtran", "Book", "Letter"];
    for template in templates.iter() {
        let option = document.create_element("option")?;
        option.set_text_content(Some(template));
        template_select.append_child(&option)?;
    }
    
    template_group.append_child(&template_label)?;
    template_group.append_child(&template_select)?;
    
    // API Selection
    let api_form_group = create_element_with_class("div", "form-group");
    let api_label = create_element_with_class("label", "form-label");
    api_label.set_text_content(Some("AI Provider"));
    
    let api_select = document.create_element("select")?.dyn_into::<web_sys::HtmlSelectElement>()?;
    api_select.set_class_name("form-select");
    api_select.set_id("api-provider");
    
    let providers = ["Claude", "Perplexity", "Mistral"];
    for provider in providers.iter() {
        let option = document.create_element("option")?;
        option.set_text_content(Some(provider));
        api_select.append_child(&option)?;
    }
    
    // Load saved API provider
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        if let Ok(Some(provider)) = storage.get_item("api_provider") {
            api_select.set_value(&provider);
        }
    }
    
    api_form_group.append_child(&api_label)?;
    api_form_group.append_child(&api_select)?;
    
    // PDF Size Selection
    let pdf_size_group = create_element_with_class("div", "form-group");
    let pdf_size_label = create_element_with_class("label", "form-label");
    pdf_size_label.set_text_content(Some("PDF Size"));
    
    let pdf_size_select = document.create_element("select")?;
    pdf_size_select.set_class_name("form-select");
    pdf_size_select.set_id("pdf-size-select");
    
    let sizes = ["Small", "Medium", "Large", "Extra Large"];
    for size in sizes.iter() {
        let option = document.create_element("option")?;
        option.set_text_content(Some(size));
        pdf_size_select.append_child(&option)?;
    }
    
    pdf_size_group.append_child(&pdf_size_label)?;
    pdf_size_group.append_child(&pdf_size_select)?;
    
    // Add all groups to options row
    options_row.append_child(&template_group)?;
    options_row.append_child(&api_form_group)?;
    options_row.append_child(&pdf_size_group)?;
    
    more_options_dropdown.append_child(&options_row)?;
    
    attachment_container.append_child(&attach_btn)?;
    attachment_container.append_child(&file_input)?;
    attachment_container.append_child(&more_options_btn)?;
    attachment_container.append_child(&more_options_dropdown)?;
    
    // Send button
    let send_btn = create_element_with_class("button", "send-btn");
    send_btn.set_id("send-btn");
    send_btn.set_inner_html(r#"
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="22" y1="2" x2="11" y2="13"></line>
            <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
        </svg>
    "#);
    
    let input_row = create_element_with_class("div", "input-row");
    input_row.append_child(&attachment_container)?;
    input_row.append_child(&chat_textarea.unchecked_ref())?;
    input_row.append_child(&send_btn)?;
    
    chat_controls.append_child(&input_row)?;
    chat_controls.append_child(&more_options_dropdown)?;
    
    chat_input_container.append_child(&chat_controls)?;
    
    left_panel.append_child(&chat_history)?;
    left_panel.append_child(&chat_input_container)?;
    
    // Right panel - Preview
    let right_panel = create_element_with_class("div", "right-panel");
    right_panel.set_attribute("style", "overflow: hidden; display: flex; flex-direction: column; background-color: hsl(var(--card)); height: 100%; position: relative;")?;
    
    // Preview header with toggle buttons
    let preview_header = create_element_with_class("div", "preview-header");
    
    let preview_title = create_element_with_class("h2", "preview-title");
    preview_title.set_text_content(Some("Document Preview"));
    
    let toggle_container = create_element_with_class("div", "toggle-container");
    
    let latex_toggle = create_element_with_class("button", "toggle-btn active");
    latex_toggle.set_id("latex-toggle");
    latex_toggle.set_text_content(Some("LaTeX"));
    
    let pdf_toggle = create_element_with_class("button", "toggle-btn");
    pdf_toggle.set_id("pdf-toggle");
    pdf_toggle.set_text_content(Some("PDF"));
    
    toggle_container.append_child(&latex_toggle)?;
    toggle_container.append_child(&pdf_toggle)?;
    
    preview_header.append_child(&preview_title)?;
    preview_header.append_child(&toggle_container)?;
    
    // Preview content
    let preview_content = create_element_with_class("div", "preview-content");
    preview_content.set_id("preview-content");
    preview_content.set_attribute("style", "flex: 1; overflow-y: auto; padding: 1rem;")?;
    preview_content.set_inner_html(r#"<div class="empty-state"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg><p>Generated content will appear here</p></div>"#);
    
    // Download button (small pill in corner)
    let download_btn = create_element_with_class("button", "download-pill");
    download_btn.set_id("download-btn");
    download_btn.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>"#);
    download_btn.set_attribute("disabled", "true")?;
    download_btn.set_attribute("style", "position: absolute; bottom: 1rem; right: 1rem; z-index: 10;")?;
    
    right_panel.append_child(&preview_header)?;
    right_panel.append_child(&preview_content)?;
    right_panel.append_child(&download_btn)?;
    
    // Append panels to main
    main.append_child(&history_panel)?;
    main.append_child(&profile_panel)?;
    main.append_child(&left_panel)?;
    main.append_child(&right_panel)?;
    
    // Assemble the UI
    container.append_child(&header)?;
    container.append_child(&main)?;
    body.append_child(&container)?;
    
    // Create Rc<RefCell<Document>> for event handlers
    let document_rc = Rc::new(RefCell::new(document.clone()));
    
    // Store generated content
    let generated_content = Rc::new(RefCell::new(None));
    
    // Store chat history
    let chat_history_state = Rc::new(RefCell::new(Vec::new()));
    
    // Store API keys
    let api_keys = Rc::new(RefCell::new(ApiKeys {
        claude: String::new(),
        perplexity: String::new(),
        mistral: String::new(),
    }));
    
    // Load saved API keys
    {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(claude_key)) = storage.get_item("claude_api_key") {
                document.get_element_by_id("claude-key").unwrap()
                    .dyn_into::<HtmlInputElement>().unwrap()
                    .set_value(&claude_key);
                api_keys.borrow_mut().claude = claude_key;
            }
            
            if let Ok(Some(perplexity_key)) = storage.get_item("perplexity_api_key") {
                document.get_element_by_id("perplexity-key").unwrap()
                    .dyn_into::<HtmlInputElement>().unwrap()
                    .set_value(&perplexity_key);
                api_keys.borrow_mut().perplexity = perplexity_key;
            }
            
            if let Ok(Some(mistral_key)) = storage.get_item("mistral_api_key") {
                document.get_element_by_id("mistral-key").unwrap()
                    .dyn_into::<HtmlInputElement>().unwrap()
                    .set_value(&mistral_key);
                api_keys.borrow_mut().mistral = mistral_key;
            }
        }
    }
    
    // Get elements for event listeners
    let send_btn = document.get_element_by_id("send-btn").unwrap();
    let download_btn = document.get_element_by_id("download-btn").unwrap();
    let chat_input = document.get_element_by_id("chat-input").unwrap().dyn_into::<HtmlTextAreaElement>()?;
    let api_select = document.get_element_by_id("api-provider").unwrap().dyn_into::<HtmlSelectElement>()?;
    let template_select = document.get_element_by_id("template-select").unwrap().dyn_into::<HtmlSelectElement>()?;
    let pdf_size_select = document.get_element_by_id("pdf-size-select").unwrap().dyn_into::<HtmlSelectElement>()?;
    let latex_toggle_element = latex_toggle.dyn_into::<HtmlElement>()?;
    let pdf_toggle_element = pdf_toggle.dyn_into::<HtmlElement>()?;
    let theme_toggle_element = theme_toggle.dyn_into::<HtmlElement>()?;
    let theme_dropdown_element = theme_dropdown.dyn_into::<HtmlElement>()?;
    let light_option_element = light_option.dyn_into::<HtmlElement>()?;
    let dark_option_element = dark_option.dyn_into::<HtmlElement>()?;
    let system_option_element = system_option.dyn_into::<HtmlElement>()?;
    let history_toggle_element = history_toggle.dyn_into::<HtmlElement>()?;
    let profile_toggle_element = profile_toggle.dyn_into::<HtmlElement>()?;
    let new_chat_btn_element = new_chat_btn.dyn_into::<HtmlElement>()?;
    let save_keys_btn = document.get_element_by_id("save-keys-btn").unwrap();
    let file_input_element = file_input.dyn_into::<HtmlInputElement>()?;
    let attach_btn_element = attach_btn.dyn_into::<HtmlElement>()?;
    let more_options_btn_element = more_options_btn.dyn_into::<HtmlElement>()?;
    let more_options_dropdown_element = more_options_dropdown.dyn_into::<HtmlElement>()?;
    let history_panel_element = history_panel.dyn_into::<HtmlElement>()?;
    let profile_panel_element = profile_panel.dyn_into::<HtmlElement>()?;
    let left_panel_element = left_panel.dyn_into::<HtmlElement>()?;
    
    // Theme toggle listener
    {
        let theme_dropdown_element = theme_dropdown_element.clone();
        let theme_toggle_callback = Closure::wrap(Box::new(move || {
            let display = theme_dropdown_element.style().get_property_value("display").unwrap();
            if display == "none" {
                theme_dropdown_element.style().set_property("display", "block").unwrap();
            } else {
                theme_dropdown_element.style().set_property("display", "none").unwrap();
            }
        }) as Box<dyn FnMut()>);
        
        theme_toggle_element.add_event_listener_with_callback("click", theme_toggle_callback.as_ref().unchecked_ref())?;
        theme_toggle_callback.forget();
    }
    
    // Theme option listeners
    {
        let document_clone = document_rc.clone();
        let theme_dropdown_element = theme_dropdown_element.clone();
        let light_callback = Closure::wrap(Box::new(move || {
            let document = document_clone.borrow();
            let body = document.body().unwrap();
            body.set_class_name("light-theme");
            theme_dropdown_element.style().set_property("display", "none").unwrap();
            
            // Save theme preference
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("theme", "light");
            }
        }) as Box<dyn FnMut()>);
        
        light_option_element.add_event_listener_with_callback("click", light_callback.as_ref().unchecked_ref())?;
        light_callback.forget();
    }
    
    {
        let document_clone = document_rc.clone();
        let theme_dropdown_element = theme_dropdown_element.clone();
        let dark_callback = Closure::wrap(Box::new(move || {
            let document = document_clone.borrow();
            let body = document.body().unwrap();
            body.set_class_name("dark-theme");
            theme_dropdown_element.style().set_property("display", "none").unwrap();
            
            // Save theme preference
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("theme", "dark");
            }
        }) as Box<dyn FnMut()>);
        
        dark_option_element.add_event_listener_with_callback("click", dark_callback.as_ref().unchecked_ref())?;
        dark_callback.forget();
    }
    
    {
        let document_clone = document_rc.clone();
        let theme_dropdown_element = theme_dropdown_element.clone();
        let system_callback = Closure::wrap(Box::new(move || {
            let document = document_clone.borrow();
            let body = document.body().unwrap();
            let window = window().unwrap();
            let media_query = window.match_media("(prefers-color-scheme: dark)").unwrap();
            
            if let Some(media_query) = media_query {
                if media_query.matches() {
                    body.set_class_name("dark-theme");
                } else {
                    body.set_class_name("light-theme");
                }
            }
            theme_dropdown_element.style().set_property("display", "none").unwrap();
            
            // Save theme preference
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("theme", "system");
            }
        }) as Box<dyn FnMut()>);
        
        system_option_element.add_event_listener_with_callback("click", system_callback.as_ref().unchecked_ref())?;
        system_callback.forget();
    }
    
    // Close dropdown when clicking outside
    {
        let document_clone = document_rc.clone();
        let theme_dropdown_element = theme_dropdown_element.clone();
        let theme_toggle_element = theme_toggle_element.clone();
        let click_callback = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Some(target) = event.target() {
                let target_node = target.dyn_into::<Node>().unwrap();
                let document = document_clone.borrow();
                
                if !theme_toggle_element.contains(Some(&target_node)) && 
                   !theme_dropdown_element.contains(Some(&target_node)) {
                    theme_dropdown_element.style().set_property("display", "none").unwrap();
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        document.add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())?;
        click_callback.forget();
    }
    
    // Load saved theme preference
    {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(theme)) = storage.get_item("theme") {
                let body = document.body().unwrap();
                match theme.as_str() {
                    "light" => {
                        body.set_class_name("light-theme");
                    },
                    "dark" => {
                        body.set_class_name("dark-theme");
                    },
                    "system" => {
                        let window = window().unwrap();
                        let media_query = window.match_media("(prefers-color-scheme: dark)").unwrap();
                        
                        if let Some(media_query) = media_query {
                            if media_query.matches() {
                                body.set_class_name("dark-theme");
                            } else {
                                body.set_class_name("light-theme");
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    // History panel toggle
    {
        let history_panel_element = history_panel_element.clone();
        let profile_panel_element = profile_panel_element.clone();
        let left_panel_element = left_panel_element.clone();
        
        let history_toggle_callback = Closure::wrap(Box::new(move || {
            if history_panel_element.class_name().contains("visible") {
                history_panel_element.set_class_name("history-panel");
                left_panel_element.set_class_name("left-panel");
            } else {
                history_panel_element.set_class_name("history-panel visible");
                profile_panel_element.set_class_name("profile-panel");
                left_panel_element.set_class_name("left-panel shifted");
            }
        }) as Box<dyn FnMut()>);
        
        history_toggle_element.add_event_listener_with_callback("click", history_toggle_callback.as_ref().unchecked_ref())?;
        history_toggle_callback.forget();
    }
    
    // Profile panel toggle
    {
        let profile_panel_element = profile_panel_element.clone();
        let history_panel_element = history_panel_element.clone();
        let left_panel_element = left_panel_element.clone();
        
        let profile_toggle_callback = Closure::wrap(Box::new(move || {
            if profile_panel_element.class_name().contains("visible") {
                profile_panel_element.set_class_name("profile-panel");
                left_panel_element.set_class_name("left-panel");
            } else {
                profile_panel_element.set_class_name("profile-panel visible");
                history_panel_element.set_class_name("history-panel");
                left_panel_element.set_class_name("left-panel shifted");
            }
        }) as Box<dyn FnMut()>);
        
        profile_toggle_element.add_event_listener_with_callback("click", profile_toggle_callback.as_ref().unchecked_ref())?;
        profile_toggle_callback.forget();
    }
    
    // Save API keys button
    {
        let api_keys = api_keys.clone();
        let api_select = api_select.clone();
        let save_keys_callback = Closure::wrap(Box::new(move || {
            let document = get_document();
            let claude_key = document.get_element_by_id("claude-key").unwrap()
                .dyn_into::<HtmlInputElement>().unwrap()
                .value();
            
            let perplexity_key = document.get_element_by_id("perplexity-key").unwrap()
                .dyn_into::<HtmlInputElement>().unwrap()
                .value();
            
            let mistral_key = document.get_element_by_id("mistral-key").unwrap()
                .dyn_into::<HtmlInputElement>().unwrap()
                .value();
            
            let api_provider = api_select.value();
            
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                let _ = storage.set_item("claude_api_key", &claude_key);
                let _ = storage.set_item("perplexity_api_key", &perplexity_key);
                let _ = storage.set_item("mistral_api_key", &mistral_key);
                let _ = storage.set_item("api_provider", &api_provider);
            }
            
            let mut keys = api_keys.borrow_mut();
            keys.claude = claude_key;
            keys.perplexity = perplexity_key;
            keys.mistral = mistral_key;
            
            alert("API keys and provider saved successfully!");
        }) as Box<dyn FnMut()>);
        
        save_keys_btn.add_event_listener_with_callback("click", save_keys_callback.as_ref().unchecked_ref())?;
        save_keys_callback.forget();
    }
    
    // File upload handler
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        
        let upload_callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let input = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
            if let Some(file_list) = input.files() {
                let file_list: FileList = file_list;
                if file_list.length() > 0 {
                    let file = file_list.get(0).unwrap();
                    let reader = web_sys::FileReader::new().unwrap();
                    
                    let document_rc = document_rc.clone();
                    let generated_content = generated_content.clone();
                    
                    let onload = Closure::wrap(Box::new(move |e: web_sys::ProgressEvent| {
                        let content = e.target().unwrap()
                            .dyn_into::<web_sys::FileReader>().unwrap()
                            .result().unwrap()
                            .as_string().unwrap();
                        
                        // Update preview with LaTeX content
                        let document = document_rc.borrow();
                        let preview_content = document.get_element_by_id("preview-content").unwrap();
                        preview_content.set_inner_html(&format!("<pre class='latex-content'>{}</pre>", content));
                        
                        // Store the uploaded content
                        let pdf_size = document.get_element_by_id("pdf-size-select").unwrap()
                            .dyn_into::<HtmlSelectElement>().unwrap()
                            .value();
                        
                        let template = document.get_element_by_id("template-select").unwrap()
                            .dyn_into::<HtmlSelectElement>().unwrap()
                            .value();
                        
                        let ai_provider = document.get_element_by_id("api-provider").unwrap()
                            .dyn_into::<HtmlSelectElement>().unwrap()
                            .value();
                        
                        *generated_content.borrow_mut() = Some(GeneratedContent {
                            latex: content,
                            pdf_blob: None,
                            pdf_url: None,
                            chat_history: Vec::new(),
                            pdf_size,
                            template,
                            ai_provider,
                        });
                        
                        // Enable download button
                        document.get_element_by_id("download-btn").unwrap()
                            .remove_attribute("disabled").unwrap();
                        
                        // Switch to LaTeX view
                        document.get_element_by_id("latex-toggle").unwrap()
                            .set_class_name("toggle-btn active");
                        document.get_element_by_id("pdf-toggle").unwrap()
                            .set_class_name("toggle-btn");
                    }) as Box<dyn FnMut(_)>);
                    
                    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget();
                    
                    reader.read_as_text(&file).unwrap();
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        file_input_element.add_event_listener_with_callback("change", upload_callback.as_ref().unchecked_ref())?;
        upload_callback.forget();
    }
    
    // Attach button click handler
    {
        let file_input_element = file_input_element.clone();
        let attach_callback = Closure::wrap(Box::new(move || {
            file_input_element.click();
        }) as Box<dyn FnMut()>);
        
        attach_btn_element.add_event_listener_with_callback("click", attach_callback.as_ref().unchecked_ref())?;
        attach_callback.forget();
    }
    
    // More options button click handler
    {
        let more_options_dropdown_element = more_options_dropdown_element.clone();
        let more_options_callback = Closure::wrap(Box::new(move || {
            let style = more_options_dropdown_element.style();
            if style.get_property_value("display").unwrap() == "none" {
                style.set_property("display", "block").unwrap();
            } else {
                style.set_property("display", "none").unwrap();
            }
        }) as Box<dyn FnMut()>);
    
        more_options_btn_element.add_event_listener_with_callback("click", more_options_callback.as_ref().unchecked_ref())?;
        more_options_callback.forget();
    }
    
    // Close more options when clicking outside
    {
        let document_clone = document_rc.clone();
        let click_callback = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let document = document_clone.borrow();
            let more_options_dropdown = document.get_element_by_id("more-options-dropdown").unwrap()
                .dyn_into::<HtmlElement>().unwrap();
            let more_options_btn = document.get_element_by_id("more-options-btn").unwrap()
                .dyn_into::<HtmlElement>().unwrap();
        
            if let Some(target) = event.target() {
                let target_node = target.dyn_into::<Node>().unwrap();
                if !more_options_btn.contains(Some(&target_node)) && 
                   !more_options_dropdown.contains(Some(&target_node)) {
                    more_options_dropdown.style()
                        .set_property("display", "none")
                        .unwrap();
                }
            }
        }) as Box<dyn FnMut(_)>);
    
        document.add_event_listener_with_callback("click", click_callback.as_ref().unchecked_ref())?;
        click_callback.forget();
    }

    // New chat button
    {
        let document_rc = document_rc.clone();
        let new_chat_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
        
            // Clear chat input
            document.get_element_by_id("chat-input").unwrap()
                .dyn_into::<HtmlTextAreaElement>().unwrap()
                .set_value("");
            
            // Clear chat history display
            document.get_element_by_id("chat-history").unwrap()
                .set_inner_html(r#"
                    <div class="empty-state">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1-2-2V5a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 1-2-2z"></path>
                        </svg>
                        <p>Start a new conversation</p>
                    </div>
                "#);
            
            // Clear preview
            document.get_element_by_id("preview-content").unwrap()
                .set_inner_html(r#"
                    <div class="empty-state">
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                            <polyline points="14 2 14 8 20 8"></polyline>
                            <line x1="16" y1="13" x2="8" y2="13"></line>
                            <line x1="16" y1="17" x2="8" y2="17"></line>
                            <polyline points="10 9 9 9 8 9"></polyline>
                        </svg>
                        <p>Generated content will appear here</p>
                    </div>
                "#);
            
            // Disable download button
            document.get_element_by_id("download-btn").unwrap()
                .set_attribute("disabled", "true").unwrap();
            
            // Hide more options dropdown
            document.get_element_by_id("more-options-dropdown").unwrap()
                        .dyn_into::<HtmlElement>().unwrap()
                        .style()
                        .set_property("display", "none")
                        .unwrap();
                }) as Box<dyn FnMut()>);
    
                new_chat_btn_element.add_event_listener_with_callback("click", new_chat_callback.as_ref().unchecked_ref())?;
                new_chat_callback.forget();
            }
    
    // Send button listener
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        let chat_history_state = chat_history_state.clone();
        let api_keys = api_keys.clone();
        let api_select = api_select.clone();
        let pdf_size_select = pdf_size_select.clone();
        let template_select = template_select.clone();
        
        let send_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            let api_provider = api_select.value();
            let template = template_select.value();
            let pdf_size = pdf_size_select.value();
            
            let topic = document.get_element_by_id("chat-input").unwrap()
                .dyn_into::<HtmlTextAreaElement>().unwrap()
                .value();
            
            if topic.is_empty() {
                alert("Please enter a topic");
                return;
            }
            
            let api_key = match api_provider.as_str() {
                "Claude" => api_keys.borrow().claude.clone(),
                "Perplexity" => api_keys.borrow().perplexity.clone(),
                "Mistral" => api_keys.borrow().mistral.clone(),
                _ => String::new()
            };
            
            if api_key.is_empty() {
                alert(&format!("Please enter your {} API key in the profile settings", api_provider));
                return;
            }
            
            // Update UI to show loading state
            document.get_element_by_id("send-btn").unwrap()
                .set_attribute("disabled", "true").unwrap();
            
            let chat_history = document.get_element_by_id("chat-history").unwrap();
            
            // Clear empty state if present
            if chat_history.query_selector(".empty-state").unwrap().is_some() {
                chat_history.set_inner_html("");
            }
            
            // Add user message to chat history
            let user_message = document.create_element("div").unwrap();
            user_message.set_class_name("chat-message user-message");
            user_message.set_inner_html(&format!(
                r#"<div class="message-content">{}</div>"#,
                topic.replace("\n", "<br>")
            ));
            chat_history.append_child(&user_message).unwrap();
            
            // Add loading indicator for AI response
            let ai_message = document.create_element("div").unwrap();
            ai_message.set_class_name("chat-message ai-message");
            ai_message.set_inner_html(r#"
                <div class="message-content">
                    <div class="loader-spinner small"></div>
                </div>
            "#);
            chat_history.append_child(&ai_message).unwrap();
            
            // Scroll to bottom of chat
            chat_history.scroll_with_x_and_y(0.0, chat_history.scroll_height() as f64);
            
            // Update preview to show loading state
            document.get_element_by_id("preview-content").unwrap()
                .set_inner_html(r#"<div class="loader"><div class="loader-spinner"></div><p>Generating document...</p></div>"#);
            
            // Hide more options dropdown
            document.get_element_by_id("more-options-dropdown").unwrap()
                .dyn_into::<HtmlElement>().unwrap()
                .style()
                .set_property("display", "none")
                .unwrap();
    
            wasm_bindgen_futures::spawn_local({
                let document_rc = document_rc.clone();
                let generated_content = generated_content.clone();
                let chat_history_state = chat_history_state.clone();
                let api_provider = api_provider.clone();
                let api_key = api_key.clone();
                let template = template.clone();
                let topic = topic.clone();
                let pdf_size = pdf_size.clone();
                
                async move {
                    match generate_latex_content(&api_provider, &api_key, &topic, &template).await {
                        Ok(content) => {
                            // Store the generated content
                            let mut chat_history = Vec::new();
                            chat_history.push(("user".to_string(), topic.clone()));
                            chat_history.push(("ai".to_string(), content.clone()));
                            
                            *generated_content.borrow_mut() = Some(GeneratedContent {
                                latex: content.clone(),
                                pdf_blob: None,
                                pdf_url: None,
                                chat_history,
                                pdf_size: pdf_size.clone(),
                                template: template.clone(),
                                ai_provider: api_provider.clone(),
                            });
                            
                            // Update preview with LaTeX content
                            let preview_content = document_rc.borrow().get_element_by_id("preview-content").unwrap();
                            preview_content.set_inner_html(&format!("<pre class='latex-content'>{}</pre>", content));
                            
                            // Update AI message in chat
                            let chat_history_element = document_rc.borrow().get_element_by_id("chat-history").unwrap();
                            if let Some(last_message) = chat_history_element.last_child() {
                                let last_message = last_message.dyn_into::<Element>().unwrap();
                                last_message.set_inner_html(&format!(
                                    r#"<div class="message-content">
                                        <div>Generated LaTeX document with {} sections</div>
                                        <div class="message-meta">
                                            <span>Template: {}</span>
                                            <span>AI: {}</span>
                                            <span>Size: {}</span>
                                        </div>
                                    </div>"#,
                                    content.matches(r"\section").count(),
                                    template,
                                    api_provider,
                                    pdf_size
                                ));
                            }
                            
                            // Add to chat history state
                            let now = js_sys::Date::new_0();
                            let date_str = now.to_locale_date_string("en-US", &JsValue::UNDEFINED);
                            let time_str = now.to_locale_time_string("en-US");                            

                            let mut history = chat_history_state.borrow_mut();
                            history.push((date_str.as_string().unwrap(), time_str.as_string().unwrap(), topic.clone(), content.clone()));
                            
                            // Update history panel
                            update_history_panel(&document_rc.borrow(), &history);
                            
                            // Enable download button
                            document_rc.borrow().get_element_by_id("download-btn").unwrap()
                                .remove_attribute("disabled").unwrap();
                            
                            // Re-enable send button
                            document_rc.borrow().get_element_by_id("send-btn").unwrap()
                                .remove_attribute("disabled").unwrap();
                            
                            // Clear input
                            document_rc.borrow().get_element_by_id("chat-input").unwrap()
                                .dyn_into::<HtmlTextAreaElement>().unwrap()
                                .set_value("");
                        },
                        Err(err) => {
                            let error_msg = format!("Error: {}", err.as_string().unwrap_or_else(|| "Unknown error".to_string()));
                            document_rc.borrow().get_element_by_id("preview-content").unwrap()
                                .set_inner_html(&format!("<div class='error'>{}</div>", error_msg));
                            
                            // Update AI message with error
                            let chat_history = document_rc.borrow().get_element_by_id("chat-history").unwrap();
                            if let Some(last_message) = chat_history.last_child() {
                                let last_message = last_message.dyn_into::<Element>().unwrap();
                                last_message.set_inner_html(&format!(
                                    r#"<div class="message-content error">Failed to generate document: {}</div>"#,
                                    error_msg
                                ));
                            }
                            
                            // Re-enable send button
                            document_rc.borrow().get_element_by_id("send-btn").unwrap()
                                .remove_attribute("disabled").unwrap();
                        }
                    }
                }
            });
        }) as Box<dyn FnMut()>);
        
        send_btn.add_event_listener_with_callback("click", send_callback.as_ref().unchecked_ref())?;
        send_callback.forget();
    }
    
    // Chat input enter key listener
    {
        let document_rc = document_rc.clone();
        let chat_input_callback = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == "Enter" && !event.shift_key() {
                event.prevent_default();
                let document = document_rc.borrow();
                let send_btn = document.get_element_by_id("send-btn").unwrap();
                if !send_btn.has_attribute("disabled") {
                    send_btn.dyn_into::<web_sys::HtmlElement>().unwrap().click();
                }
            }
        });
        
        chat_input.add_event_listener_with_callback("keydown", chat_input_callback.as_ref().unchecked_ref())?;
        chat_input_callback.forget();
    }  

    // Download button listener
    {
        let document_rc = document_rc.clone();
        let generated_content = Rc::clone(&generated_content); // Use Rc::clone instead of moving
        
        let download_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            
            if let Some(content) = &*generated_content.borrow() {
                // Show compilation in progress with a white background
                document.get_element_by_id("preview-content").unwrap()
                    .set_inner_html(r#"
                        <div class="pdf-message" style="background: white; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center;">
                            <div class="loader-spinner"></div>
                            <p>Compiling PDF for download...</p>
                        </div>
                    "#);
                
                // In a real implementation, you would:
                // 1. Send the LaTeX content to a compilation service
                // 2. Receive the PDF binary data
                // 3. Create a blob and download link
                
                // For now, we'll simulate this with a timeout
                let window = web_sys::window().unwrap();
                let document_rc = document_rc.clone();
                let generated_content = Rc::clone(&generated_content); // Clone again for the closure
                
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    Closure::once_into_js(move || {
                        // In a real implementation, replace this with actual PDF compilation
                        let mut content = generated_content.borrow_mut();
                        if let Some(content) = &mut *content {
                            // Create a simple PDF with the text "Compiled PDF"
                            let pdf_content = format!("%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n5 0 obj\n<< /Length 44 >>\nstream\nBT\n/F1 12 Tf\n100 700 Td\n(Compiled PDF) Tj\nET\nendstream\nendobj\nxref\n0 6\n0000000000 65535 f \n0000000010 00000 n \n0000000079 00000 n \n0000000173 00000 n \n0000000301 00000 n \n0000000380 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n492\n%%EOF");
                            
                            // Convert to Uint8Array
                            let mut pdf_bytes = Vec::with_capacity(pdf_content.len());
                            pdf_bytes.extend_from_slice(pdf_content.as_bytes());
                            let pdf_array = Uint8Array::from(&pdf_bytes[..]);
                            
                            let blob_parts = Array::new();
                            blob_parts.push(&pdf_array);
                            
                            match Blob::new_with_u8_array_sequence(&blob_parts) {
                                Ok(blob) => {
                                    content.pdf_blob = Some(blob);
                                    
                                    match Url::create_object_url_with_blob(content.pdf_blob.as_ref().unwrap()) {
                                        Ok(pdf_url) => {
                                            content.pdf_url = Some(pdf_url.clone());
                                            
                                            // Create download link for PDF
                                            let doc = document_rc.borrow();
                                            let a = doc.create_element("a").unwrap();
                                            let a = a.dyn_into::<web_sys::HtmlElement>().unwrap();
                                            a.set_attribute("href", &pdf_url).unwrap();
                                            a.set_attribute("download", "document.pdf").unwrap();
                                            a.set_attribute("style", "display: none").unwrap();
                                            
                                            doc.body().unwrap().append_child(&a).unwrap();
                                            a.click();
                                            doc.body().unwrap().remove_child(&a).unwrap();
                                        },
                                        Err(e) => {
                                            console::error_1(&JsString::from(format!("Failed to create object URL: {:?}", e)));
                                        }
                                    }
                                },
                                Err(e) => {
                                    console::error_1(&JsString::from(format!("Failed to create blob: {:?}", e)));
                                }
                            }
                        }
                    }).unchecked_ref(),
                    1000, // Simulate 1 second compilation time
                );
            } else {
                alert("No content generated yet.");
            }
        }) as Box<dyn FnMut()>);
        
        download_btn.add_event_listener_with_callback("click", download_callback.as_ref().unchecked_ref())?;
        download_callback.forget();
    }
    
    // Toggle buttons listeners
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        
        let latex_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            
            if let Some(content) = &*generated_content.borrow() {
                document.get_element_by_id("preview-content").unwrap()
                    .set_inner_html(&format!("<pre class='latex-content'>{}</pre>", content.latex));
            }
            
            document.get_element_by_id("latex-toggle").unwrap()
                .set_class_name("toggle-btn active");
            document.get_element_by_id("pdf-toggle").unwrap()
                .set_class_name("toggle-btn");
        }) as Box<dyn FnMut()>);
        
        latex_toggle_element.add_event_listener_with_callback("click", latex_callback.as_ref().unchecked_ref())?;
        latex_callback.forget();
    }
    
    // PDF toggle callback
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        let window = web_sys::window().unwrap();
    
        let pdf_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            let preview_content = document.get_element_by_id("preview-content").unwrap();
        
            if let Some(content) = &mut *generated_content.borrow_mut() {
                // Clean up any existing URL
                if let Some(url) = content.pdf_url.take() {
                    Url::revoke_object_url(&url).ok();
                }
            
                // Show compilation in progress with white background
                preview_content.set_inner_html(r#"
                    <div class="pdf-message" style="background: white; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center;">
                        <div class="loader-spinner"></div>
                        <p>Compiling PDF preview...</p>
                    </div>
                "#);
            
                // Simulate PDF compilation (in a real app, you'd call a LaTeX compilation service)
                let document_rc = document_rc.clone();
                let content_clone = generated_content.clone();
            
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    Closure::once_into_js(move || {
                        let mut content = content_clone.borrow_mut();
                        if let Some(content) = &mut *content {
                            // Here you would normally send the LaTeX to a compilation service
                            // For this example, we'll create a simple PDF with the LaTeX content rendered as text
                        
                            // Create a PDF with the LaTeX content
                            let latex_content = &content.latex;
                            let pdf_content = format!("%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n5 0 obj\n<< /Length {} >>\nstream\nBT\n/F1 12 Tf\n100 700 Td\n({}) Tj\nET\nendstream\nendobj\nxref\n0 6\n0000000000 65535 f \n0000000010 00000 n \n0000000079 00000 n \n0000000173 00000 n \n0000000301 00000 n \n0000000380 00000 n \ntrailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n492\n%%EOF", 
                                latex_content.len() + 20, // Length placeholder
                                latex_content.replace("(", "\\(").replace(")", "\\)") // Escape parentheses
                            );
                        
                            // Convert to Uint8Array
                            let mut pdf_bytes = Vec::with_capacity(pdf_content.len());
                            pdf_bytes.extend_from_slice(pdf_content.as_bytes());
                            let pdf_array = Uint8Array::from(&pdf_bytes[..]);
                        
                            let blob_parts = Array::new();
                            blob_parts.push(&pdf_array);
                        
                            match Blob::new_with_u8_array_sequence(&blob_parts) {
                                Ok(blob) => {
                                    content.pdf_blob = Some(blob);
                                
                                    match Url::create_object_url_with_blob(content.pdf_blob.as_ref().unwrap()) {
                                        Ok(pdf_url) => {
                                            content.pdf_url = Some(pdf_url.clone());
                                        
                                            let doc = document_rc.borrow();
                                            let preview_content = doc.get_element_by_id("preview-content").unwrap();
                                            preview_content.set_inner_html(&format!(
                                                r#"<iframe src="{}" style="width:100%;height:100%;border:none;background:white;"></iframe>"#,
                                                pdf_url
                                            ));
                                        },
                                        Err(e) => {
                                            console::error_1(&JsString::from(format!("Failed to create object URL: {:?}", e)));
                                        }
                                    }
                                },
                                Err(e) => {
                                    console::error_1(&JsString::from(format!("Failed to create blob: {:?}", e)));
                                }
                            }
                        }
                    }).unchecked_ref(),
                    1000, // Simulate 1 second compilation time
                );
            }
        
            document.get_element_by_id("latex-toggle").unwrap()
                .set_class_name("toggle-btn");
            document.get_element_by_id("pdf-toggle").unwrap()
                .set_class_name("toggle-btn active");
        }) as Box<dyn FnMut()>);

        pdf_toggle_element.add_event_listener_with_callback("click", pdf_callback.as_ref().unchecked_ref())?;
        pdf_callback.forget();
    }
    
    // History item click handler
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        let history_list = document.get_element_by_id("history-list").unwrap();
        
        let history_click_callback = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            if let Some(target) = event.target() {
                if let Ok(element) = target.dyn_into::<Element>() {
                    let parent = element.parent_element();
                    let node_to_compare = parent.as_ref().map(|v| v as &web_sys::Node);
                    
                    if element.class_name().contains("history-entry") || 
                       element.parent_element().map_or(false, |p| p.class_name().contains("history-entry")) {
                        let document = document_rc.borrow();
                        let history_list = document.get_element_by_id("history-list").unwrap();
                        let entries = history_list.query_selector_all(".history-entry").unwrap();

                        for i in 0..entries.length() {
                            if let Ok(entry) = entries.get(i).unwrap().dyn_into::<Element>() {
                                if entry.is_same_node(Some(&element)) || 
                                   entry.is_same_node(node_to_compare) {
                                    if let Some(data_index) = entry.get_attribute("data-index") {
                                        let data_index = data_index.parse::<usize>().unwrap_or(0);
                                        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                                            if let Ok(Some(history)) = storage.get_item("chat_history_data") {
                                                if let Ok(history_data) = JSON::parse(&history) {
                                                    if let Ok(entry_data) = Reflect::get(&history_data, &JsValue::from_f64(data_index as f64)) {
                                                        let topic = Reflect::get(&entry_data, &JsValue::from_str("topic"))
                                                            .ok()
                                                            .and_then(|v| v.as_string())
                                                            .unwrap_or_default();
                    
                                                        let content = Reflect::get(&entry_data, &JsValue::from_str("content"))
                                                            .ok()
                                                            .and_then(|v| v.as_string())
                                                            .unwrap_or_default();
                                                        
                                                        let template = Reflect::get(&entry_data, &JsValue::from_str("template"))
                                                            .ok()
                                                            .and_then(|v| v.as_string())
                                                            .unwrap_or_else(|| "Article".to_string());
                                                        
                                                        let ai_provider = Reflect::get(&entry_data, &JsValue::from_str("ai_provider"))
                                                            .ok()
                                                            .and_then(|v| v.as_string())
                                                            .unwrap_or_else(|| "Claude".to_string());
                                                        
                                                        let pdf_size = Reflect::get(&entry_data, &JsValue::from_str("pdf_size"))
                                                            .ok()
                                                            .and_then(|v| v.as_string())
                                                            .unwrap_or_else(|| "Medium".to_string());
                                                        
                                                        // Update chat history display
                                                        let chat_history = document.get_element_by_id("chat-history").unwrap();
                                                        chat_history.set_inner_html("");
                                                        
                                                        let user_message = document.create_element("div").unwrap();
                                                        user_message.set_class_name("chat-message user-message");
                                                        user_message.set_inner_html(&format!(
                                                            r#"<div class="message-content">{}</div>"#,
                                                            topic.replace("\n", "<br>")
                                                        ));
                                                        chat_history.append_child(&user_message).unwrap();
                                                        
                                                        let ai_message = document.create_element("div").unwrap();
                                                        ai_message.set_class_name("chat-message ai-message");
                                                        ai_message.set_inner_html(&format!(
                                                            r#"<div class="message-content">
                                                                <div>Generated LaTeX document with {} sections</div>
                                                                <div class="message-meta">
                                                                    <span>Template: {}</span>
                                                                    <span>AI: {}</span>
                                                                    <span>Size: {}</span>
                                                                </div>
                                                            </div>"#,
                                                            content.matches(r"\section").count(),
                                                            template,
                                                            ai_provider,
                                                            pdf_size
                                                        ));
                                                        chat_history.append_child(&ai_message).unwrap();
                                                        
                                                        // Update preview with LaTeX content
                                                        let preview_content = document.get_element_by_id("preview-content").unwrap();
                                                        preview_content.set_inner_html(&format!("<pre class='latex-content'>{}</pre>", content));
                                                        
                                                        // Store the generated content
                                                        let mut chat_history = Vec::new();
                                                        chat_history.push(("user".to_string(), topic));
                                                        chat_history.push(("ai".to_string(), content.clone()));
                                                        
                                                        *generated_content.borrow_mut() = Some(GeneratedContent {
                                                            latex: content,
                                                            pdf_blob: None,
                                                            pdf_url: None,
                                                            chat_history,
                                                            pdf_size,
                                                            template,
                                                            ai_provider,
                                                        });
                                                        
                                                        // Enable download button
                                                        document.get_element_by_id("download-btn").unwrap()
                                                            .remove_attribute("disabled").unwrap();
                                                        
                                                        // Switch to LaTeX view
                                                        document.get_element_by_id("latex-toggle").unwrap()
                                                            .set_class_name("toggle-btn active");
                                                        document.get_element_by_id("pdf-toggle").unwrap()
                                                            .set_class_name("toggle-btn");
                                                        
                                                        // Close history panel
                                                        document.get_element_by_id("history-panel").unwrap()
                                                            .set_class_name("history-panel");
                                                        document.get_element_by_id("left-panel").unwrap()
                                                            .set_class_name("left-panel");
                                                        
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);
        
        history_list.add_event_listener_with_callback("click", history_click_callback.as_ref().unchecked_ref())?;
        history_click_callback.forget();
    }
    
    // Add CSS styles
    let style = document.create_element("style")?;
    style.set_text_content(Some(get_css()));
    document.query_selector("head")?.unwrap().append_child(&style)?;
    
    Ok(())
}

// Update history panel with chat entries
fn update_history_panel(document: &Document, history: &[(String, String, String, String)]) {
    let history_list = document.get_element_by_id("history-list").unwrap();
    
    // Clear existing content
    history_list.set_inner_html("");
    
    // Group by date
    let mut grouped: std::collections::BTreeMap<String, Vec<(String, String, String, String)>> = std::collections::BTreeMap::new();
    for entry in history.iter().rev() {
        grouped.entry(entry.0.clone()).or_default().push(entry.clone());
    }
    
    // Prepare data for storage
    let history_data = js_sys::Array::new();
    
    // Add entries to history panel
    for (date, entries) in grouped {
        let date_header = document.create_element("div").unwrap();
        date_header.set_class_name("history-date");
        date_header.set_text_content(Some(&date));
        history_list.append_child(&date_header).unwrap();
        
        for (i, (date, time, topic, content)) in entries.iter().enumerate() {
            let entry = document.create_element("div").unwrap();
            entry.set_class_name("history-entry");
            entry.set_attribute("data-index", &i.to_string()).unwrap();
            
            let time_span = document.create_element("span").unwrap();
            time_span.set_class_name("history-time");
            time_span.set_text_content(Some(time));
            
            let topic_span = document.create_element("span").unwrap();
            topic_span.set_class_name("history-topic");
            topic_span.set_text_content(Some(topic));
            
            entry.append_child(&time_span).unwrap();
            entry.append_child(&topic_span).unwrap();
            history_list.append_child(&entry).unwrap();
            
            // Add to history data array
            let entry_data = js_sys::Object::new();
            Reflect::set(&entry_data, &JsValue::from_str("date"), &JsValue::from_str(date)).unwrap();
            Reflect::set(&entry_data, &JsValue::from_str("time"), &JsValue::from_str(time)).unwrap();
            Reflect::set(&entry_data, &JsValue::from_str("topic"), &JsValue::from_str(topic)).unwrap();
            Reflect::set(&entry_data, &JsValue::from_str("content"), &JsValue::from_str(content)).unwrap();
            
            // Get current template, provider and size
            let template = document.get_element_by_id("template-select").unwrap()
                .dyn_into::<HtmlSelectElement>().unwrap()
                .value();
            let ai_provider = document.get_element_by_id("api-provider").unwrap()
                .dyn_into::<HtmlSelectElement>().unwrap()
                .value();
            let pdf_size = document.get_element_by_id("pdf-size-select").unwrap()
                .dyn_into::<HtmlSelectElement>().unwrap()
                .value();
            
            Reflect::set(&entry_data, &JsValue::from_str("template"), &JsValue::from_str(&template)).unwrap();
            Reflect::set(&entry_data, &JsValue::from_str("ai_provider"), &JsValue::from_str(&ai_provider)).unwrap();
            Reflect::set(&entry_data, &JsValue::from_str("pdf_size"), &JsValue::from_str(&pdf_size)).unwrap();
            
            history_data.push(&entry_data);
        }
    }
    
    // Save to local storage
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        let _ = storage.set_item("chat_history", &history_list.inner_html());
        let _ = storage.set_item("chat_history_data", &JSON::stringify(&history_data).unwrap().as_string().unwrap());
    }
}

async fn generate_latex_content(provider: &str, api_key: &str, topic: &str, template: &str) -> Result<String, JsValue> {
    let window = web_sys::window().unwrap();
    
    // Prepare API request based on provider
    let url = match provider {
        "Claude" => "https://api.anthropic.com/v1/messages",
        "Perplexity" => "https://api.perplexity.ai/chat/completions",
        "Mistral" => "https://api.mistral.ai/v1/chat/completions",
        _ => return Err(JsValue::from_str("Invalid API provider"))
    };
    
    // Prepare request headers
    let headers = Headers::new().unwrap();
    
    match provider {
        "Claude" => {
            headers.append("x-api-key", api_key).unwrap();
            headers.append("anthropic-version", "2023-06-01").unwrap();
            headers.append("content-type", "application/json").unwrap();
        },
        "Mistral" | "Perplexity" => {
            headers.append("authorization", &format!("Bearer {}", api_key)).unwrap();
            headers.append("content-type", "application/json").unwrap();
        },
        _ => {}
    }
    
    // Prepare the prompt based on template
    let (doc_class, additional_packages) = match template {
        "IEEEtran" => ("IEEEtran", r#"\usepackage[utf8]{inputenc}\usepackage{amsmath}\usepackage{graphicx}\usepackage{hyperref}\usepackage{cite}\usepackage{amsfonts}\usepackage{amssymb}\usepackage{url}"#),
        "Report" => ("report", r#"\usepackage[utf8]{inputenc}\usepackage{amsmath}\usepackage{graphicx}\usepackage{hyperref}\usepackage{titlesec}"#),
        "Book" => ("book", r#"\usepackage[utf8]{inputenc}\usepackage{amsmath}\usepackage{graphicx}\usepackage{hyperref}\usepackage{fancyhdr}"#),
        "Letter" => ("letter", r#"\usepackage[utf8]{inputenc}\usepackage{hyperref}\usepackage{geometry}"#),
        _ => ("article", r#"\usepackage[utf8]{inputenc}\usepackage{amsmath}\usepackage{graphicx}\usepackage{hyperref}"#)
    };
    
    let prompt = format!(
        "Generate a comprehensive LaTeX document about '{}' using the '{}' document class. Include appropriate sections, equations, and references. Format it as a complete LaTeX document that can be compiled directly. Use these packages:\n\n{}\n\nMake sure to include:\n\n1. A title section\n2. At least 3 content sections\n3. At least one equation\n4. Proper document structure with begin/end document\n5. All necessary template-specific elements for {}",
        topic, doc_class, additional_packages, doc_class
    );
    
    // Prepare request body based on provider
    let body = match provider {
        "Claude" => {
            serde_json::json!({
                "model": "claude-3-opus-20240229",
                "max_tokens": 4000,
                "messages": [{"role": "user", "content": prompt}]
            })
        },
        "Mistral" => {
            serde_json::json!({
                "model": "mistral-large-latest",
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.7
            })
        },
        "Perplexity" => {
            serde_json::json!({
                "model": "pplx-7b-online",
                "messages": [{"role": "user", "content": prompt}]
            })
        },
        _ => return Err(JsValue::from_str("Invalid API provider"))
    };
    
    let body_str = body.to_string();


    let mut request_init = RequestInit::new();
    request_init.set_method("POST");
    request_init.set_headers(&headers);
    request_init.set_body(JsValue::from_str(&body_str).as_ref());
    
    // Send request
    let request = web_sys::Request::new_with_str_and_init(url, &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response = response.dyn_into::<Response>()?;
    
    // Check status
    if !response.ok() {
        let status = response.status();
        let status_text = response.status_text();
        let error_body = JsFuture::from(response.text()?).await?;
        let error_msg = format!("API error: {} {}\n{}", status, status_text, error_body.as_string().unwrap_or_default());
        return Err(JsValue::from_str(&error_msg));
    }
    
    // Parse response
    let json = JsFuture::from(response.json()?).await?;
    
    // Extract content based on provider
    let content = match provider {
        "Claude" => {
            let content = Reflect::get(&json, &JsValue::from_str("content"))?;
            let content_array = js_sys::Array::from(&content);
            if content_array.length() > 0 {
                let first_content = content_array.get(0);
                Reflect::get(&first_content, &JsValue::from_str("text"))?.as_string().unwrap_or_default()
            } else {
                return Err(JsValue::from_str("Invalid response format from Claude API"));
            }
        },
        "Mistral" | "Perplexity" => {
            let choices = Reflect::get(&json, &JsValue::from_str("choices"))?;
            let choices_array = js_sys::Array::from(&choices);
            if choices_array.length() > 0 {
                let first_choice = choices_array.get(0);
                let message = Reflect::get(&first_choice, &JsValue::from_str("message"))?;
                Reflect::get(&message, &JsValue::from_str("content"))?.as_string().unwrap_or_default()
            } else {
                return Err(JsValue::from_str("Invalid response format from API"));
            }
        },
        _ => return Err(JsValue::from_str("Invalid API provider"))
    };
    
    // Extract LaTeX code from the content
    let latex_content = extract_latex_document(&content);
    
    Ok(latex_content)
}

// Function to extract LaTeX document from AI response
fn extract_latex_document(content: &str) -> String {
    // Try to find content between \documentclass and \end{document}
    if let Some(start_idx) = content.find("\\documentclass") {
        if let Some(end_idx) = content.find("\\end{document}") {
            return content[start_idx..=end_idx + 13].to_string();
        }
    }
    
    // If not found, look for content in markdown code blocks
    if let Some(start_idx) = content.find("```latex") {
        let start_pos = start_idx + 8;
        if let Some(end_idx) = content[start_pos..].find("```") {
            return content[start_pos..start_pos + end_idx].trim().to_string();
        }
    } else if let Some(start_idx) = content.find("```tex") {
        let start_pos = start_idx + 6;
        if let Some(end_idx) = content[start_pos..].find("```") {
            return content[start_pos..start_pos + end_idx].trim().to_string();
        }
    }
    
    // If no specific LaTeX formatting found, return the whole content
    content.to_string()
}

// CSS Styles
fn get_css() -> &'static str {
    r#"
    :root {
        --background: 0 0% 100%;
        --foreground: 240 10% 3.9%;
        --card: 0 0% 100%;
        --card-foreground: 240 10% 3.9%;
        --popover: 0 0% 100%;
        --popover-foreground: 240 10% 3.9%;
        --primary: 142.1 76.2% 36.3%;
        --primary-foreground: 355.7 100% 97.3%;
        --secondary: 240 4.8% 95.9%;
        --secondary-foreground: 240 5.9% 10%;
        --muted: 240 4.8% 95.9%;
        --muted-foreground: 240 3.8% 46.1%;
        --accent: 240 4.8% 95.9%;
        --accent-foreground: 240 5.9% 10%;
        --destructive: 0 84.2% 60.2%;
        --destructive-foreground: 0 0% 98%;
        --border: 240 5.9% 90%;
        --input: 240 5.9% 90%;
        --ring: 142.1 76.2% 36.3%;
        --radius: 0.5rem;

        --zinc-50: 240 4.8% 95.9%;
        --zinc-100: 240 5% 96.1%;
        --zinc-200: 240 5.9% 90%;
        --zinc-300: 240 4.9% 83.9%;
        --zinc-400: 240 5% 64.9%;
        --zinc-500: 240 4% 46.1%;
        --zinc-600: 240 5.2% 33.9%;
        --zinc-700: 240 5.3% 26.1%;
        --zinc-800: 240 3.7% 15.9%;
        --zinc-900: 240 5.2% 11.9%;
        --zinc-950: 240 3.7% 7.5%;

        --bg-default: 0 0% 100%; /* White in light mode */
        --bg-basic-gray-subtle: var(--zinc-100);
    }

    .dark-theme {
        --background: 240 3.7% 7.5%; /* zinc-950 - darker background */
        --foreground: 0 0% 95%;
        --card: 240 3.7% 15.9%; /* zinc-800 */
        --card-foreground: 0 0% 95%;
        --popover: 240 3.7% 15.9%; /* zinc-800 */
        --popover-foreground: 0 0% 95%;
        --primary: 142.1 70.6% 45.3%;
        --primary-foreground: 144.9 80.4% 10%;
        --secondary: 240 3.7% 15.9%; /* zinc-800 */
        --secondary-foreground: 0 0% 95%;
        --muted: 240 3.7% 15.9%; /* zinc-800 */
        --muted-foreground: 240 5% 70.9%;
        --accent: 240 3.7% 15.9%; /* zinc-800 */
        --accent-foreground: 0 0% 95%;
        --destructive: 0 62.8% 40.6%;
        --destructive-foreground: 0 0% 98%;
        --border: 240 3.7% 23.9%; /* between zinc-800 and zinc-700 */
        --input: 240 3.7% 15.9%; /* zinc-800 */
        --ring: 142.1 70.6% 45.3%;

        --bg-default: var(--zinc-950); /* Zinc-950 in dark mode */
        --bg-basic-gray-subtle: var(--zinc-800);
    }

    * {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    html, body {
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 0;
        overflow: hidden;
        background-color: hsl(var(--bg-default));
    }

    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
        background-color: hsl(var(--bg-default));
        color: hsl(var(--foreground));
        line-height: 1.6;
        transition: background-color 0.3s, color 0.3s;
    }

    .container {
        width: 100vw;
        height: 100vh;
        margin: 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        background-color: hsl(var(--bg-default));
    }

    .app-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1.5rem;
        border-bottom: 1px solid hsl(var(--border));
        background-color: hsl(var(--card));
        box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
        height: 60px;
    }

    .logo-container {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .logo-img {
        height: 3rem;
        width: auto;
    }

    .header-buttons {
        display: flex;
        gap: 0.5rem;
    }

    .header-btn {
        background: none;
        border: none;
        color: hsl(var(--foreground));
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 0.375rem;
        transition: background-color 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 2.5rem;
        height: 2.5rem;
    }

    .header-btn:hover {
        background-color: hsl(var(--muted));
    }

    .header-btn svg {
        width: 1.25rem;
        height: 1.25rem;
    }

    .theme-container {
        position: relative;
    }

    .theme-dropdown {
        position: absolute;
        top: 100%;
        right: 0;
        background: hsl(var(--card));
        border: 1px solid hsl(var(--border));
        border-radius: 0.5rem;
        padding: 0.5rem;
        z-index: 100;
        display: none;
        min-width: 120px;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
    }

    .theme-dropdown.visible {
        display: block;
    }

    .theme-option {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        width: 100%;
        padding: 0.5rem;
        background: none;
        border: none;
        color: hsl(var(--foreground));
        cursor: pointer;
        border-radius: 0.25rem;
        font-size: 0.875rem;
    }

    .theme-option:hover {
        background-color: hsl(var(--accent));
    }

    .theme-option svg {
        width: 1rem;
        height: 1rem;
    }

    .main-content {
        width: 100%;
        height: calc(100vh - 60px);
        display: grid;
        grid-template-columns: 280px 1fr;
        gap: 0;
        min-height: 0;
        overflow: hidden;
        position: relative;
        background-color: hsl(var(--bg-default));
    }

    .history-panel, .profile-panel {
        background-color: hsl(var(--card));
        border-right: 1px solid hsl(var(--border));
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
        transform: translateX(-100%);
        transition: transform 0.3s ease;
        position: absolute;
        top: 0;
        left: 0;
        bottom: 0;
        width: 480px;
        z-index: 10;
    }

    .history-panel.visible, .profile-panel.visible {
        transform: translateX(0);
    }

    .history-header, .profile-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .history-header h3, .profile-header h3 {
        margin: 0;
        font-size: 1rem;
        font-weight: 600;
    }

    #new-chat-btn {
        margin-bottom: 1rem;
    }

    .history-list {
        flex: 1;
        overflow-y: auto;
    }

    .history-date {
        font-size: 0.875rem;
        font-weight: 500;
        color: hsl(var(--muted-foreground));
        margin: 1rem 0 0.5rem 0;
    }

    .history-entry {
        padding: 0.5rem 0.75rem;
        border-radius: 0.375rem;
        margin-bottom: 0.5rem;
        cursor: pointer;
        transition: background-color 0.2s;
        background-color: hsl(var(--bg-basic-gray-subtle));
    }

    .history-entry:hover {
        background-color: hsl(var(--accent));
    }

    .history-time {
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
        margin-right: 0.5rem;
    }

    .history-topic {
        font-size: 0.875rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: block;
    }

    .left-panel, .right-panel {
        overflow: hidden;
        display: flex;
        flex-direction: column;
        background-color: hsl(var(--bg-default));
        border-right: 1px solid hsl(var(--border));
        height: 100%;
        transition: transform 0.3s ease;
    }

    .left-panel.shifted {
        transform: translateX(280px);
    }

    .right-panel {
        border-right: none;
    }

    .chat-history {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        background-color: hsl(var(--bg-default));
    }

    .chat-message {
        max-width: 85%;
        padding: 0.75rem 1rem;
        border-radius: 0.75rem;
        line-height: 1.5;
        font-size: 0.9375rem;
        box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    }

    .user-message {
        align-self: flex-end;
        background-color: hsl(var(--primary) / 0.1);
        border-bottom-right-radius: 0.25rem;
        color: hsl(var(--foreground));
    }

    .ai-message {
        align-self: flex-start;
        background-color: hsl(var(--bg-basic-gray-subtle));
        border-bottom-left-radius: 0.25rem;
        color: hsl(var(--foreground));
    }

    .message-content {
        word-break: break-word;
    }

    .message-meta {
        display: flex;
        gap: 0.75rem;
        margin-top: 0.5rem;
        font-size: 0.75rem;
        color: hsl(var(--muted-foreground));
    }

    .message-content.error {
        color: hsl(var(--destructive));
    }

    .chat-input-container {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 1rem;
        border-top: 1px solid hsl(var(--border));
        background-color: hsl(var(--card));
    }

    .input-row {
        display: flex;
        align-items: flex-end;
        gap: 0.5rem;
        position: relative;
    }

    .chat-textarea {
        flex: 1;
        min-height: 6.25rem;
        max-height: 200px;
        padding: 0.75rem 3rem 0.75rem 1rem;
        border: 1px solid hsl(var(--input));
        border-radius: 0.5rem;
        background-color: hsl(var(--bg-basic-gray-subtle));
        color: hsl(var(--foreground));
        font-size: 0.9375rem;
        resize: none;
        transition: border-color 0.2s, box-shadow 0.2s;
        font-family: inherit;
        box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
        overflow-y: auto;
    }

    .chat-textarea:focus {
        outline: none;
        border-color: hsl(var(--primary));
        box-shadow: 0 0 0 3px hsl(var(--primary) / 0.2);
    }

    .attachment-container {
        display: flex;
        gap: 0.5rem;
        position: absolute;
        bottom: 0.75rem;
        left: 0.75rem;
    }

    .attach-btn, .more-options-btn, .send-btn {
        background: none;
        border: none;
        color: hsl(var(--foreground));
        cursor: pointer;
        padding: 0.5rem;
        border-radius: 0.375rem;
        transition: background-color 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 2.5rem;
        height: 2.5rem;
    }

    .attach-btn:hover, .more-options-btn:hover, .send-btn:hover {
        background-color: hsl(var(--muted));
    }

    .send-btn {
        background-color: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
        position: absolute;
        bottom: 0.75rem;
        right: 0.75rem;
    }

    .send-btn:hover {
        background-color: hsl(var(--primary) / 0.9);
    }

    .send-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .attach-btn svg, .more-options-btn svg, .send-btn svg {
        width: 1.25rem;
        height: 1.25rem;
    }

    .more-options-dropdown {
        background-color: hsl(var(--card));
        border: 1px solid hsl(var(--border));
        border-radius: 0.5rem;
        padding: 1rem;
        margin-top: 0.5rem;
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
        width: 100%;
    }

    .options-row {
        display: flex;
        gap: 1rem;
        width: 100%;
    }

    .form-group {
        flex: 1;
        margin-bottom: 0;
    }

    .form-label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 500;
        font-size: 0.875rem;
        color: hsl(var(--foreground));
    }

    .form-input, .form-select {
        width: 100%;
        padding: 0.5rem 0.75rem;
        border: 1px solid hsl(var(--input));
        border-radius: 0.375rem;
        background-color: hsl(var(--bg-basic-gray-subtle));
        color: hsl(var(--foreground));
        font-size: 0.875rem;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    .form-input:focus, .form-select:focus {
        outline: none;
        border-color: hsl(var(--primary));
        box-shadow: 0 0 0 3px hsl(var(--primary) / 0.2);
    }

    .btn-primary, .btn-secondary {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        padding: 0.625rem 1rem;
        border: none;
        border-radius: 0.375rem;
        font-size: 0.9375rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 0.2s, transform 0.1s;
        grid-column: span 2;
    }

    .btn-primary {
        background-color: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
    }

    .btn-primary:hover {
        background-color: hsl(var(--primary) / 0.9);
    }

    .btn-primary:active {
        transform: scale(0.98);
    }

    .btn-primary:disabled {
        background-color: hsl(var(--muted-foreground));
        cursor: not-allowed;
        transform: none;
        opacity: 0.6;
    }

    .btn-secondary {
        background-color: hsl(var(--bg-basic-gray-subtle));
        color: hsl(var(--foreground));
    }

    .btn-secondary:hover {
        background-color: hsl(var(--accent));
    }

    .btn-secondary:active {
        transform: scale(0.98);
    }

    .btn-secondary:disabled {
        opacity: 0.6;
        cursor: not-allowed;
        transform: none;
    }

    .preview-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 1rem;
        border-bottom: 1px solid hsl(var(--border));
        background-color: hsl(var(--card));
    }

    .preview-title {
        margin: 0;
        font-size: 1rem;
        font-weight: 600;
    }

    .toggle-container {
        display: flex;
        gap: 0.25rem;
        background-color: hsl(var(--bg-basic-gray-subtle));
        padding: 0.25rem;
        border-radius: 0.375rem;
    }

    .toggle-btn {
        padding: 0.375rem 0.75rem;
        border: none;
        background: none;
        color: hsl(var(--foreground));
        border-radius: 0.25rem;
        cursor: pointer;
        font-size: 0.875rem;
        font-weight: 500;
        transition: background-color 0.2s, color 0.2s;
    }

    .toggle-btn.active {
        background-color: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
    }

    .preview-content {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
        background-color: hsl(var(--bg-default));
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: hsl(var(--muted-foreground));
        text-align: center;
        padding: 2.5rem 1.25rem;
        background-color: hsl(var(--bg-default));
    }

    .empty-state svg {
        width: 3rem;
        height: 3rem;
        margin-bottom: 1rem;
        color: hsl(var(--muted-foreground));
    }

    .empty-state p {
        margin: 0;
        font-size: 0.9375rem;
    }

    .latex-content {
        white-space: pre-wrap;
        font-family: 'Courier New', Courier, monospace;
        font-size: 0.875rem;
        line-height: 1.5;
        margin: 0;
        padding: 1rem;
        background-color: hsl(var(--card));
        border-radius: 0.5rem;
        border: 1px solid hsl(var(--border));
        overflow-x: auto;
    }

    .loader {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 2.5rem 1.25rem;
        background-color: hsl(var(--bg-default));
    }

    .loader-spinner {
        width: 2.5rem;
        height: 2.5rem;
        border: 0.25rem solid hsl(var(--primary) / 0.2);
        border-top-color: hsl(var(--primary));
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 1rem;
    }

    .loader-spinner.small {
        width: 1.25rem;
        height: 1.25rem;
        border-width: 0.125rem;
        margin-bottom: 0;
    }

    .loader p {
        margin: 0;
        color: hsl(var(--muted-foreground));
        font-size: 0.9375rem;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    .error {
        color: hsl(var(--destructive));
        padding: 1rem;
        background-color: hsl(var(--destructive) / 0.1);
        border-radius: 0.5rem;
        font-size: 0.9375rem;
    }

    .pdf-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 2.5rem 1.25rem;
        text-align: center;
        color: hsl(var(--foreground));
        background: white;
    }

    .pdf-message svg {
        width: 3rem;
        height: 3rem;
        margin-bottom: 1rem;
        color: hsl(var(--primary));
    }

    .pdf-message h3 {
        margin: 0 0 0.5rem 0;
        font-size: 1.125rem;
    }

    .pdf-message p {
        margin: 0 0 0.5rem 0;
        font-size: 0.9375rem;
        color: hsl(var(--muted-foreground));
        max-width: 25rem;
    }

    .preview-content iframe {
        width: 100%;
        height: 100%;
        border: none;
        background: white;
    }

    .api-keys-form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .download-pill {
        width: 2.5rem;
        height: 2.5rem;
        border-radius: 50%;
        background-color: hsl(var(--primary));
        color: hsl(var(--primary-foreground));
        border: none;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        transition: transform 0.2s, background-color 0.2s;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    }

    .download-pill:hover {
        background-color: hsl(var(--primary) / 0.9);
        transform: scale(1.05);
    }

    .download-pill:active {
        transform: scale(0.95);
    }

    .download-pill:disabled {
        opacity: 0.5;
        cursor: not-allowed;
        transform: none !important;
    }

    .download-pill svg {
        width: 1rem;
        height: 1rem;
    }

    @media (max-width: 1024px) {
        .history-panel, .profile-panel {
            width: 240px;
        }
        
        .main-content {
            grid-template-columns: 240px 1fr;
        }
    }

    @media (max-width: 768px) {
        .main-content {
            grid-template-columns: 1fr;
            grid-template-rows: auto auto;
        }
        
        .left-panel {
            grid-row: 2;
            border-right: none;
            border-bottom: 1px solid hsl(var(--border));
        }
        
        .right-panel {
            grid-row: 1;
        }

        .chat-controls {
            grid-template-columns: 1fr;
        }

        .btn-primary, .btn-secondary {
            grid-column: span 1;
        }

        .options-row {
            flex-direction: column;
        }
    }
    "#
}
