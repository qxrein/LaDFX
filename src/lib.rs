use wasm_bindgen::prelude::*;
use web_sys::{window, Document, HtmlElement, HtmlTextAreaElement, HtmlSelectElement, HtmlInputElement, Element, Headers, Blob, Url};
use js_sys::{Array, JsString, Uint8Array, Reflect};
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen_futures::JsFuture;
use web_sys::console;

// Structure to store generated content
struct GeneratedContent {
    latex: String,
    pdf_blob: Option<Blob>,
    pdf_url: Option<String>,
}

// Initialize console error panic hook for better debugging
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Helper function to get document
fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

// Helper function to create an element with a class
fn create_element_with_class(tag: &str, class_name: &str) -> HtmlElement {
    let element = document().create_element(tag).unwrap();
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
    let document = document();
    let body = document.body().unwrap();
    
    // Add theme class to body
    body.set_class_name("dark-theme");
    
    // Create the UI container
    let container = create_element_with_class("div", "container");
    
    // Header with logo and toggle buttons
    let header = create_element_with_class("div", "app-header");
    
    // Logo and history toggle button
    let logo_container = create_element_with_class("div", "logo-container");
    
    let history_toggle = create_element_with_class("button", "history-toggle");
    history_toggle.set_id("history-toggle");
    history_toggle.set_inner_html(r#"<img src="logotex.png" alt="LaTeX AI Logo" class="logo-img">"#);
    
    logo_container.append_child(&history_toggle)?;
    header.append_child(&logo_container)?;
    
    // Theme toggle
    let theme_toggle = create_element_with_class("button", "theme-toggle");
    theme_toggle.set_id("theme-toggle");
    theme_toggle.set_inner_html(r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>"#);
    
    header.append_child(&theme_toggle)?;
    
    // Main content area
    let main = create_element_with_class("div", "main-content");
    
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
                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
                    </svg>
                    <p>No chat history yet</p>
                </div>
            "#);
        }
    }
    
    history_panel.append_child(&history_header)?;
    history_panel.append_child(&new_chat_btn)?;
    history_panel.append_child(&history_list)?;
    
    // Left panel - Chat and Settings
    let left_panel = create_element_with_class("div", "left-panel");
    
    // Chat history
    let chat_history = create_element_with_class("div", "chat-history");
    chat_history.set_id("chat-history");
    chat_history.set_inner_html(r#"
        <div class="empty-state">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
            </svg>
            <p>Your chat history will appear here</p>
        </div>
    "#);
    
    // Chat input container
    let chat_input_container = create_element_with_class("div", "chat-input-container");
    
    // Chat textarea
    let chat_textarea = document.create_element("textarea")?;
    chat_textarea.set_class_name("chat-textarea");
    chat_textarea.set_id("chat-input");
    chat_textarea.set_attribute("placeholder", "Describe the LaTeX document you want to generate...")?;
    chat_textarea.set_attribute("rows", "4")?;
    let chat_textarea = chat_textarea.dyn_into::<HtmlTextAreaElement>()?;
    
    // Chat controls
    let chat_controls = create_element_with_class("div", "chat-controls");
    
    // API Selection
    let api_form_group = create_element_with_class("div", "form-group api-group");
    let api_label = create_element_with_class("label", "form-label");
    api_label.set_text_content(Some("AI Provider"));
    
    let api_select = document.create_element("select")?;
    api_select.set_class_name("form-select");
    api_select.set_id("api-provider");
    
    let providers = ["No API", "Claude", "Perplexity", "Mistral"];
    for provider in providers.iter() {
        let option = document.create_element("option")?;
        option.set_text_content(Some(provider));
        api_select.append_child(&option)?;
    }
    
    api_form_group.append_child(&api_label)?;
    api_form_group.append_child(&api_select)?;
    
    // API Key
    let key_form_group = create_element_with_class("div", "form-group api-group");
    key_form_group.set_id("api-key-group");
    let key_label = create_element_with_class("label", "form-label");
    key_label.set_text_content(Some("API Key"));
    
    let key_input = document.create_element("input")?;
    key_input.set_class_name("form-input");
    key_input.set_id("api-key");
    let key_input = key_input.dyn_into::<HtmlInputElement>()?;
    key_input.set_type("password");
    key_input.set_placeholder("sk-...");
    
    key_form_group.append_child(&key_label)?;
    key_form_group.append_child(&key_input.unchecked_ref())?;
    
    // Generate button
    let submit_btn = create_element_with_class("button", "btn-primary");
    submit_btn.set_id("generate-btn");
    submit_btn.set_text_content(Some("Generate LaTeX"));
    
    chat_controls.append_child(&api_form_group)?;
    chat_controls.append_child(&key_form_group)?;
    chat_controls.append_child(&submit_btn)?;
    
    chat_input_container.append_child(&chat_textarea.unchecked_ref())?;
    chat_input_container.append_child(&chat_controls)?;
    
    left_panel.append_child(&chat_history)?;
    left_panel.append_child(&chat_input_container)?;
    
    // Right panel - Preview
    let right_panel = create_element_with_class("div", "right-panel");
    
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
    preview_content.set_inner_html(r#"<div class="empty-state"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg><p>Generated content will appear here</p></div>"#);
    
    // Download button
    let download_btn = create_element_with_class("button", "btn-secondary");
    download_btn.set_id("download-btn");
    download_btn.set_text_content(Some("Download PDF"));
    download_btn.set_attribute("disabled", "true")?;
    
    right_panel.append_child(&preview_header)?;
    right_panel.append_child(&preview_content)?;
    right_panel.append_child(&download_btn)?;
    
    // Append panels to main
    main.append_child(&history_panel)?;
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
    
    // Get elements for event listeners
    let generate_btn = document.get_element_by_id("generate-btn").unwrap();
    let download_btn = document.get_element_by_id("download-btn").unwrap();
    let chat_input = document.get_element_by_id("chat-input").unwrap().dyn_into::<HtmlTextAreaElement>()?;
    let api_select_element = api_select.dyn_into::<HtmlSelectElement>()?;
    let _key_form_group_element = key_form_group.dyn_into::<HtmlElement>()?;
    let latex_toggle_element = latex_toggle.dyn_into::<HtmlElement>()?;
    let pdf_toggle_element = pdf_toggle.dyn_into::<HtmlElement>()?;
    let theme_toggle_element = theme_toggle.dyn_into::<HtmlElement>()?;
    let key_input_element = key_input.dyn_into::<HtmlInputElement>()?;
    let history_toggle_element = history_toggle.dyn_into::<HtmlElement>()?;
    let new_chat_btn_element = new_chat_btn.dyn_into::<HtmlElement>()?;
    let history_panel_element = history_panel.dyn_into::<HtmlElement>()?;
    let left_panel_element = left_panel.dyn_into::<HtmlElement>()?;
    
    // Show/hide API key input based on provider selection
    {
        let document_rc = document_rc.clone();
        let api_select_element_clone = api_select_element.clone();
        let api_select_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            let provider = api_select_element_clone.value();
            let api_key_group = document.get_element_by_id("api-key-group").unwrap();
            
            if provider == "No API" {
                api_key_group.set_attribute("style", "display: none").unwrap();
            } else {
                api_key_group.remove_attribute("style").unwrap();
            }
        }) as Box<dyn FnMut()>);
        
        api_select_element.add_event_listener_with_callback("change", api_select_callback.as_ref().unchecked_ref())?;
        api_select_callback.forget();
    }
    
    // Load saved API key
    {
        let provider = api_select_element.value();
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            if let Ok(Some(saved_key)) = storage.get_item(&format!("{}_api_key", provider)) {
                key_input_element.set_value(&saved_key);
            }
        }
    }
    
    // Theme toggle listener
    {
        let document_clone = document.clone();
        let theme_toggle_callback = Closure::wrap(Box::new(move || {
            let body = document_clone.body().unwrap();
            if body.class_name() == "light-theme" {
                body.set_class_name("dark-theme");
            } else {
                body.set_class_name("light-theme");
            }
        }) as Box<dyn FnMut()>);
        
        theme_toggle_element.add_event_listener_with_callback("click", theme_toggle_callback.as_ref().unchecked_ref())?;
        theme_toggle_callback.forget();
    }
    
    // History panel toggle
    {
        let history_panel_element = history_panel_element.clone();
        let left_panel_element = left_panel_element.clone();
        
        let history_toggle_callback = Closure::wrap(Box::new(move || {
            if history_panel_element.class_name().contains("visible") {
                history_panel_element.set_class_name("history-panel");
                left_panel_element.set_class_name("left-panel");
            } else {
                history_panel_element.set_class_name("history-panel visible");
                left_panel_element.set_class_name("left-panel shifted");
            }
        }) as Box<dyn FnMut()>);
        
        history_toggle_element.add_event_listener_with_callback("click", history_toggle_callback.as_ref().unchecked_ref())?;
        history_toggle_callback.forget();
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
                            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
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
        }) as Box<dyn FnMut()>);
        
        new_chat_btn_element.add_event_listener_with_callback("click", new_chat_callback.as_ref().unchecked_ref())?;
        new_chat_callback.forget();
    }
    
    // Generate button listener
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        let chat_history_state = chat_history_state.clone();
        
        let generate_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            let api_provider = document.get_element_by_id("api-provider").unwrap()
                .dyn_into::<HtmlSelectElement>().unwrap()
                .value();
            
            let api_key = document.get_element_by_id("api-key").unwrap()
                .dyn_into::<HtmlInputElement>().unwrap()
                .value();
            
            let topic = document.get_element_by_id("chat-input").unwrap()
                .dyn_into::<HtmlTextAreaElement>().unwrap()
                .value();
            
            if topic.is_empty() {
                alert("Please enter a topic");
                return;
            }
            
            if api_provider != "No API" && api_key.is_empty() {
                alert("Please enter your API key");
                return;
            }

            if !api_key.is_empty() {
                if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                    let _ = storage.set_item(&format!("{}_api_key", api_provider), &api_key);
                }
            }
            
            // Update UI to show loading state
            document.get_element_by_id("generate-btn").unwrap()
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
            
            // Start the generation process
            wasm_bindgen_futures::spawn_local({
                let document_rc = document_rc.clone();
                let generated_content = generated_content.clone();
                let chat_history_state = chat_history_state.clone();
                
                async move {
                    match generate_latex_content(&api_provider, &api_key, &topic).await {
                        Ok(content) => {
                            // Store the generated content
                            *generated_content.borrow_mut() = Some(GeneratedContent {
                                latex: content.clone(),
                                pdf_blob: None,
                                pdf_url: None,
                            });
                            
                            // Update preview with LaTeX content
                            let preview_content = document_rc.borrow().get_element_by_id("preview-content").unwrap();
                            preview_content.set_inner_html(&format!("<pre class='latex-content'>{}</pre>", content));
                            
                            // Update AI message in chat
                            let chat_history = document_rc.borrow().get_element_by_id("chat-history").unwrap();
                            if let Some(last_message) = chat_history.last_child() {
                                let last_message = last_message.dyn_into::<Element>().unwrap();
                                last_message.set_inner_html(&format!(
                                    r#"<div class="message-content">Generated LaTeX document with {} sections</div>"#,
                                    content.matches(r"\section").count()
                                ));
                            }
                            
                            // Add to chat history state
                            let now = js_sys::Date::new_0();
                            let date_str = now.to_locale_date_string("en-US", &JsValue::UNDEFINED);
                            let time_str = now.to_locale_time_string("en-US");                            

                            let mut history = chat_history_state.borrow_mut();
                            history.push((date_str.as_string().unwrap(), time_str.as_string().unwrap(), topic.clone()));
                            
                            // Update history panel
                            update_history_panel(&document_rc.borrow(), &history);
                            
                            // Enable download button
                            document_rc.borrow().get_element_by_id("download-btn").unwrap()
                                .remove_attribute("disabled").unwrap();
                            
                            // Re-enable generate button
                            document_rc.borrow().get_element_by_id("generate-btn").unwrap()
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
                            
                            // Re-enable generate button
                            document_rc.borrow().get_element_by_id("generate-btn").unwrap()
                                .remove_attribute("disabled").unwrap();
                        }
                    }
                }
            });
        }) as Box<dyn FnMut()>);
        
        generate_btn.add_event_listener_with_callback("click", generate_callback.as_ref().unchecked_ref())?;
        generate_callback.forget();
    }
    
    // Chat input enter key listener
    {
        let document_rc = document_rc.clone();
        let chat_input_callback = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(move |event: web_sys::KeyboardEvent| {
            if event.key() == "Enter" && !event.shift_key() {
                event.prevent_default();
                let document = document_rc.borrow();
                let generate_btn = document.get_element_by_id("generate-btn").unwrap();
                if !generate_btn.has_attribute("disabled") {
                    generate_btn.dyn_into::<web_sys::HtmlElement>().unwrap().click();
                }
            }
        });
        
        chat_input.add_event_listener_with_callback("keydown", chat_input_callback.as_ref().unchecked_ref())?;
        chat_input_callback.forget();
    }  

    // Download button listener
    {
        let document_rc = document_rc.clone();
        let generated_content = generated_content.clone();
        
        let download_callback = Closure::wrap(Box::new(move || {
            let document = document_rc.borrow();
            
            if let Some(content) = &*generated_content.borrow() {
                if let Some(pdf_url) = &content.pdf_url {
                    // Create download link for PDF
                    let a = document.create_element("a").unwrap();
                    let a = a.dyn_into::<web_sys::HtmlElement>().unwrap();
                    a.set_attribute("href", pdf_url).unwrap();
                    a.set_attribute("download", "document.pdf").unwrap();
                    a.set_attribute("style", "display: none").unwrap();
                    
                    document.body().unwrap().append_child(&a).unwrap();
                    a.click();
                    document.body().unwrap().remove_child(&a).unwrap();
                } else {
                    alert("PDF is not ready yet. Please wait for compilation to finish.");
                }
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
                
                if let Some(blob) = &content.pdf_blob {
                    // Create new object URL
                    match Url::create_object_url_with_blob(blob) {
                        Ok(pdf_url) => {
                            content.pdf_url = Some(pdf_url.clone());
                            
                            // Show PDF in iframe
                            preview_content.set_inner_html(&format!(
                                r#"<iframe src="{}" style="width:100%;height:100%;border:none;"></iframe>"#,
                                pdf_url
                            ));
                        },
                        Err(e) => {
                            console::error_1(&JsString::from(format!("Failed to create object URL: {:?}", e)));
                            preview_content.set_inner_html(r#"
                                <div class="error">
                                    Failed to create PDF preview
                                </div>
                            "#);
                        }
                    }
                } else {
                    // Show compilation in progress
                    preview_content.set_inner_html(r#"
                        <div class="pdf-message">
                            <div class="loader-spinner"></div>
                            <p>Compiling PDF preview...</p>
                        </div>
                    "#);
                    
                    // Simulate PDF compilation
                    let document_clone = document_rc.clone();
                    let content_clone = generated_content.clone();
                    
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        Closure::once_into_js(move || {
                            let mut content = content_clone.borrow_mut();
                            if let Some(content) = &mut *content {
                                // Simulate creating a PDF blob
                                let pdf_data = Uint8Array::new_with_length(0); // Empty in simulation
                                let blob_parts = Array::new();
                                blob_parts.push(&pdf_data);
                                match Blob::new_with_u8_array_sequence(&blob_parts) {
                                    Ok(blob) => {
                                        content.pdf_blob = Some(blob);
                                        
                                        match Url::create_object_url_with_blob(content.pdf_blob.as_ref().unwrap()) {
                                            Ok(pdf_url) => {
                                                content.pdf_url = Some(pdf_url.clone());
                                                
                                                let doc = document_clone.borrow();
                                                let preview_content = doc.get_element_by_id("preview-content").unwrap();
                                                preview_content.set_inner_html(&format!(
                                                    r#"<iframe src="{}" style="width:100%;height:100%;border:none;"></iframe>"#,
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
                        2000, // Simulate 2 second compilation time
                    );
                }
            }
            
            document.get_element_by_id("latex-toggle").unwrap()
                .set_class_name("toggle-btn");
            document.get_element_by_id("pdf-toggle").unwrap()
                .set_class_name("toggle-btn active");
        }) as Box<dyn FnMut()>);

        pdf_toggle_element.add_event_listener_with_callback("click", pdf_callback.as_ref().unchecked_ref())?;
        pdf_callback.forget();
    }
    
    // Add CSS styles
    let style = document.create_element("style")?;
    style.set_text_content(Some(get_css()));
    document.document_element().unwrap()
        .dyn_into::<web_sys::HtmlElement>().unwrap()
        .query_selector("head").unwrap().unwrap()
        .append_child(&style)?;
    
    Ok(())
}

// Update history panel with chat entries
fn update_history_panel(document: &Document, history: &[(String, String, String)]) {
    let history_list = document.get_element_by_id("history-list").unwrap();
    
    // Clear existing content
    history_list.set_inner_html("");
    
    // Group by date
    let mut grouped: std::collections::BTreeMap<String, Vec<(String, String)>> = std::collections::BTreeMap::new();
    for (date, time, topic) in history.iter().rev() {
        grouped.entry(date.clone()).or_default().push((time.clone(), topic.clone()));
    }
    
    // Add entries to history panel
    for (date, entries) in grouped {
        let date_header = document.create_element("div").unwrap();
        date_header.set_class_name("history-date");
        date_header.set_text_content(Some(&date));
        history_list.append_child(&date_header).unwrap();
        
        for (time, topic) in entries {
            let entry = document.create_element("div").unwrap();
            entry.set_class_name("history-entry");
            
            let time_span = document.create_element("span").unwrap();
            time_span.set_class_name("history-time");
            time_span.set_text_content(Some(&time));
            
            let topic_span = document.create_element("span").unwrap();
            topic_span.set_class_name("history-topic");
            topic_span.set_text_content(Some(&topic));
            
            entry.append_child(&time_span).unwrap();
            entry.append_child(&topic_span).unwrap();
            history_list.append_child(&entry).unwrap();
        }
    }
    
    // Save to local storage
    if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
        let _ = storage.set_item("chat_history", &history_list.inner_html());
    }
}

// API call to generate LaTeX content
async fn generate_latex_content(provider: &str, api_key: &str, topic: &str) -> Result<String, JsValue> {
    if provider == "No API" {
        // Generate basic LaTeX template without API
        let latex_template = format!(
            r#"\documentclass{{article}}
\usepackage[utf8]{{inputenc}}
\usepackage{{amsmath}}
\usepackage{{graphicx}}
\usepackage{{hyperref}}

\title{{{}}}
\author{{Your Name}}
\date{{\today}}

\begin{{document}}

\maketitle

\section{{Introduction}}
This is a basic LaTeX document about {}. You can expand it with more sections, equations, and references as needed.

\section{{Main Content}}
Add your main content here. This could include:

\begin{{itemize}}
    \item Key concepts
    \item Mathematical formulas like $E = mc^2$
    \item Figures and tables
\end{{itemize}}

\begin{{equation}}
    \label{{eq:example}}
    f(x) = \int_{{-\infty}}^{{\infty}} \hat{{f}}(\xi)\,e^{{2 \pi i \xi x}} \, d\xi
\end{{equation}}

\section{{Conclusion}}
Summarize your document here.

\end{{document}}"#,
            topic, topic
        );
        return Ok(latex_template);
    }

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
        "Mistral" => {
            headers.append("authorization", &format!("Bearer {}", api_key)).unwrap();
            headers.append("content-type", "application/json").unwrap();
            headers.append("accept", "application/json").unwrap();
        },
        _ => {
            headers.append("authorization", &format!("Bearer {}", api_key)).unwrap();
            headers.append("content-type", "application/json").unwrap();
        }
    }
    
    // Prepare the prompt
    let prompt = format!(
        "Generate a comprehensive LaTeX document about '{}'. Include appropriate sections, equations, and references. Format it as a complete LaTeX document that can be compiled directly. Use the article document class and include common packages like amsmath, graphicx, and hyperref. Make sure to include:\n\n1. A title section\n2. At least 3 content sections\n3. At least one equation\n4. Proper document structure with begin/end document",
        topic
    );
    
    // Prepare request body as a string
    let body_str = match provider {
        "Claude" => {
            let escaped_prompt = prompt.replace('"', r#"\""#).replace('\n', r#"\n"#);
            format!(r#"{{
                "model": "claude-3-opus-20240229",
                "max_tokens": 4000,
                "messages": [{{"role": "user", "content": "{}"}}]
            }}"#, escaped_prompt)
        },
        "Mistral" => {
            let escaped_prompt = prompt.replace('"', r#"\""#).replace('\n', r#"\n"#);
            format!(r#"{{
                "model": "mistral-large-latest",
                "messages": [{{"role": "user", "content": "{}"}}],
                "temperature": 0.7
            }}"#, escaped_prompt)
        },
        "Perplexity" => {
            let escaped_prompt = prompt.replace('"', r#"\""#).replace('\n', r#"\n"#);
            format!(r#"{{
                "model": "pplx-7b-online",
                "messages": [{{"role": "user", "content": "{}"}}]
            }}"#, escaped_prompt)
        },
        _ => return Err(JsValue::from_str("Invalid API provider"))
    };

    // Create request
    let mut request_init = web_sys::RequestInit::new();
    request_init.set_method("POST");
    let js_body = JsValue::from_str(&body_str);
    request_init.body(Some(&js_body));
    request_init.set_headers(&headers);
    
    // Send request
    let request = web_sys::Request::new_with_str_and_init(url, &request_init)?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response = response.dyn_into::<web_sys::Response>()?;
    
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
    let content = if provider == "Claude" {
        // Claude response format
        let content = Reflect::get(&json, &JsValue::from_str("content"))?;
        let content_array = js_sys::Array::from(&content);
        
        if content_array.length() > 0 {
            let first_content = content_array.get(0);
            Reflect::get(&first_content, &JsValue::from_str("text"))?.as_string().unwrap_or_default()
        } else {
            return Err(JsValue::from_str("Invalid response format from Claude API"));
        }
    } else if provider == "Mistral" {
        // Mistral response format
        let choices = Reflect::get(&json, &JsValue::from_str("choices"))?;
        let choices_array = js_sys::Array::from(&choices);
        
        if choices_array.length() > 0 {
            let first_choice = choices_array.get(0);
            let message = Reflect::get(&first_choice, &JsValue::from_str("message"))?;
            Reflect::get(&message, &JsValue::from_str("content"))?.as_string().unwrap_or_default()
        } else {
            return Err(JsValue::from_str("Invalid response format from Mistral API"));
        }
    } else {
        // Perplexity response format
        let choices = Reflect::get(&json, &JsValue::from_str("choices"))?;
        let choices_array = js_sys::Array::from(&choices);
        
        if choices_array.length() > 0 {
            let first_choice = choices_array.get(0);
            let message = Reflect::get(&first_choice, &JsValue::from_str("message"))?;
            Reflect::get(&message, &JsValue::from_str("content"))?.as_string().unwrap_or_default()
        } else {
            return Err(JsValue::from_str("Invalid response format from Perplexity API"));
        }
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
        --primary-color: #14532d;
        --primary-hover: #052e16;
        --primary-light: #dcfce7;
        --text-color: #f8fafc;
        --text-secondary: #94a3b8;
        --bg-color: #0f172a;
        --panel-bg: #1e293b;
        --panel-border: #334155;
        --border-color: #334155;
        --input-bg: #1e293b;
        --input-border: #334155;
        --toggle-bg: #334155;
        --toggle-active: #14532d;
        --error-color: #ef4444;
        --user-message-bg: #1e3a8a;
        --ai-message-bg: #1e293b;
        --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
        --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06);
        --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
        --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05);
        --shadow-xl: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    }

    .light-theme {
        --primary-color: #14532d;
        --primary-hover: #052e16;
        --primary-light: #dcfce7;
        --text-color: #1e293b;
        --text-secondary: #64748b;
        --bg-color: #f8fafc;
        --panel-bg: #ffffff;
        --panel-border: #e2e8f0;
        --border-color: #e2e8f0;
        --input-bg: #ffffff;
        --input-border: #e2e8f0;
        --toggle-bg: #f1f5f9;
        --toggle-active: #14532d;
        --error-color: #ef4444;
        --user-message-bg: #dbeafe;
        --ai-message-bg: #f1f5f9;
    }

    * {
        box-sizing: border-box;
    }

    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
        margin: 0;
        padding: 0;
        background-color: var(--bg-color);
        color: var(--text-color);
        line-height: 1.6;
        transition: background-color 0.3s, color 0.3s;
        height: 100vh;
        overflow: hidden;
    }

    .container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 0;
        height: 100vh;
        display: flex;
        flex-direction: column;
    }

    .app-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 24px;
        border-bottom: 1px solid var(--border-color);
        background-color: var(--panel-bg);
        box-shadow: var(--shadow-sm);
    }

    .logo-container {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .logo-img {
        height: 48px;
        width: auto;
    }

    .history-toggle {
        background: none;
        border: none;
        padding: 0;
        cursor: pointer;
        display: flex;
        align-items: center;
    }

    .theme-toggle {
        background: none;
        border: none;
        color: var(--text-color);
        cursor: pointer;
        padding: 8px;
        border-radius: 6px;
        transition: background-color 0.2s;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .theme-toggle:hover {
        background-color: var(--toggle-bg);
    }

    .theme-toggle svg {
        width: 20px;
        height: 20px;
    }

    .main-content {
        display: grid;
        grid-template-columns: 280px 1fr 1fr;
        gap: 0;
        flex: 1;
        min-height: 0;
        overflow: hidden;
        position: relative;
    }

    .history-panel {
        background-color: var(--panel-bg);
        border-right: 1px solid var(--border-color);
        overflow-y: auto;
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 16px;
        transform: translateX(-100%);
        transition: transform 0.3s ease;
        position: absolute;
        top: 0;
        left: 0;
        bottom: 0;
        width: 280px;
        z-index: 10;
    }

    .history-panel.visible {
        transform: translateX(0);
    }

    .history-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .history-header h3 {
        margin: 0;
        font-size: 1rem;
        font-weight: 600;
    }

    #new-chat-btn {
        margin-bottom: 16px;
    }

    .history-list {
        flex: 1;
        overflow-y: auto;
    }

    .history-date {
        font-size: 0.875rem;
        font-weight: 500;
        color: var(--text-secondary);
        margin: 16px 0 8px 0;
    }

    .history-entry {
        padding: 8px 12px;
        border-radius: 6px;
        margin-bottom: 8px;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .history-entry:hover {
        background-color: var(--toggle-bg);
    }

    .history-time {
        font-size: 0.75rem;
        color: var(--text-secondary);
        margin-right: 8px;
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
        background-color: var(--panel-bg);
        border-right: 1px solid var(--border-color);
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
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .chat-message {
        max-width: 85%;
        padding: 12px 16px;
        border-radius: 12px;
        line-height: 1.5;
        font-size: 0.9375rem;
        box-shadow: var(--shadow-sm);
    }

    .user-message {
        align-self: flex-end;
        background-color: var(--user-message-bg);
        border-bottom-right-radius: 4px;
        color: var(--text-color);
    }

    .ai-message {
        align-self: flex-start;
        background-color: var(--ai-message-bg);
        border-bottom-left-radius: 4px;
        color: var(--text-color);
    }

    .message-content {
        word-break: break-word;
    }

    .message-content.error {
        color: var(--error-color);
    }

    .chat-input-container {
        display: flex;
        flex-direction: column;
        gap: 16px;
        padding: 16px;
        border-top: 1px solid var(--border-color);
        background-color: var(--panel-bg);
    }

    .chat-textarea {
        width: 100%;
        min-height: 100px;
        padding: 12px;
        border: 1px solid var(--input-border);
        border-radius: 8px;
        background-color: var(--input-bg);
        color: var(--text-color);
        font-size: 0.9375rem;
        resize: none;
        transition: border-color 0.2s, box-shadow 0.2s;
        font-family: inherit;
        box-shadow: var(--shadow-sm);
    }

    .chat-textarea:focus {
        outline: none;
        border-color: var(--primary-color);
        box-shadow: 0 0 0 3px rgba(21, 83, 45, 0.2);
    }

    .chat-controls {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 12px;
    }

    .form-group {
        margin: 0;
    }

    .form-label {
        display: block;
        margin-bottom: 8px;
        font-weight: 500;
        font-size: 0.875rem;
        color: var(--text-color);
    }

    .form-input, .form-select {
        width: 100%;
        padding: 8px 12px;
        border: 1px solid var(--input-border);
        border-radius: 6px;
        background-color: var(--input-bg);
        color: var(--text-color);
        font-size: 0.875rem;
        transition: border-color 0.2s, box-shadow 0.2s;
    }

    .form-input:focus, .form-select:focus {
        outline: none;
        border-color: var(--primary-color);
        box-shadow: 0 0 0 3px rgba(21, 83, 45, 0.2);
    }

    .btn-primary, .btn-secondary {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        padding: 10px 16px;
        border: none;
        border-radius: 6px;
        font-size: 0.9375rem;
        font-weight: 500;
        cursor: pointer;
        transition: background-color 0.2s, transform 0.1s;
        grid-column: span 2;
    }

    .btn-primary {
        background-color: var(--primary-color);
        color: white;
    }

    .btn-primary:hover {
        background-color: var(--primary-hover);
    }

    .btn-primary:active {
        transform: scale(0.98);
    }

    .btn-primary:disabled {
        background-color: var(--text-secondary);
        cursor: not-allowed;
        transform: none;
    }

    .btn-secondary {
        background-color: var(--toggle-bg);
        color: var(--text-color);
    }

    .btn-secondary:hover {
        background-color: var(--border-color);
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
        padding: 16px;
        border-bottom: 1px solid var(--border-color);
    }

    .preview-title {
        margin: 0;
        font-size: 1rem;
        font-weight: 600;
    }

    .toggle-container {
        display: flex;
        gap: 4px;
        background-color: var(--toggle-bg);
        padding: 4px;
        border-radius: 6px;
    }

    .toggle-btn {
        padding: 6px 12px;
        border: none;
        background: none;
        color: var(--text-color);
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.875rem;
        font-weight: 500;
        transition: background-color 0.2s, color 0.2s;
    }

    .toggle-btn.active {
        background-color: var(--toggle-active);
        color: white;
    }

    .preview-content {
        flex: 1;
        overflow-y: auto;
        padding: 16px;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: var(--text-secondary);
        text-align: center;
        padding: 40px 20px;
    }

    .empty-state svg {
        width: 48px;
        height: 48px;
        margin-bottom: 16px;
        color: var(--text-secondary);
    }

    .empty-state p {
        margin: 0;
        font-size: 0.9375rem;
    }

    .latex-content {
        white-space: pre-wrap;
        font-family: 'Courier New', Courier, monospace;
        font-size: 14px;
        line-height: 1.5;
        margin: 0;
        padding: 16px;
        background-color: var(--input-bg);
        border-radius: 8px;
        border: 1px solid var(--border-color);
        overflow-x: auto;
    }

    .loader {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 40px 20px;
    }

    .loader-spinner {
        width: 40px;
        height: 40px;
        border: 4px solid rgba(21, 83, 45, 0.2);
        border-top-color: var(--primary-color);
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin-bottom: 16px;
    }

    .loader-spinner.small {
        width: 20px;
        height: 20px;
        border-width: 2px;
        margin-bottom: 0;
    }

    .loader p {
        margin: 0;
        color: var(--text-secondary);
        font-size: 0.9375rem;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }

    .error {
        color: var(--error-color);
        padding: 16px;
        background-color: rgba(239, 68, 68, 0.1);
        border-radius: 8px;
        font-size: 0.9375rem;
    }

    .pdf-message {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        padding: 40px 20px;
        text-align: center;
        color: var(--text-color);
    }

    .pdf-message svg {
        width: 48px;
        height: 48px;
        margin-bottom: 16px;
        color: var(--primary-color);
    }

    .pdf-message h3 {
        margin: 0 0 8px 0;
        font-size: 1.125rem;
    }

    .pdf-message p {
        margin: 0 0 8px 0;
        font-size: 0.9375rem;
        color: var(--text-secondary);
        max-width: 400px;
    }

    .preview-content iframe {
        width: 100%;
        height: 100%;
        border: none;
        background: white;
    }

    @media (max-width: 1024px) {
        .history-panel {
            width: 240px;
        }
        
        .main-content {
            grid-template-columns: 240px 1fr 1fr;
        }
    }

    @media (max-width: 768px) {
        .main-content {
            grid-template-columns: 1fr;
        }

        .left-panel {
            border-right: none;
            border-bottom: 1px solid var(--border-color);
        }

        .chat-controls {
            grid-template-columns: 1fr;
        }

        .btn-primary, .btn-secondary {
            grid-column: span 1;
        }
    }
    "#
}
