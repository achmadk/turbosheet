use chromiumoxide::browser::Browser;
fn test(mut handler: chromiumoxide::browser::BrowserHandler) {
    let _ = async move {
        while let Some(event) = futures::StreamExt::next(&mut handler).await {
            match event {
                Ok(ev) => {
                    let _e: chromiumoxide::handler::Event = ev;
                }
                _ => {}
            }
        }
    };
}
