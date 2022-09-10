use actix_web::error::ErrorInternalServerError;
use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Result};
use serde_json::json;
use tinytemplate::TinyTemplate;
use youtube_dl::YoutubeDl;
use youtube_dl::YoutubeDlOutput;

static TEMPLATE: &str = r#"
<html>
<head>

<meta name="og:site_name"       content="TikTok">
<meta property="og:url"         content="{ tiktok_url }">
<meta property="og:title"       content="{ video_title }">
<meta property="og:type"        content="video.other">
<meta property="og:video"       content="{ video_url }">
<meta property="og:video:type"  content="video/mp4">

<meta http-equiv="refresh" content="0; url = { tiktok_url }" />
</head>
<body>
<a href="{ tiktok_url }">Click here if you don't get redirected.</a>
</body>
</html>
"#;

#[get("/{user_id}/video/{video_id}")]
async fn greet(
    path: web::Path<(String, String)>,
    tmpl: web::Data<TinyTemplate<'_>>,
) -> Result<HttpResponse> {
    let (user_id, video_id) = path.into_inner();
    let tiktok_url = format!("https://www.tiktok.com/{user_id}/video/{video_id}");

    let output = YoutubeDl::new(&tiktok_url)
        .socket_timeout("15")
        .run_async()
        .await
        .map_err(|_e| ErrorInternalServerError(format!("yt-dl error: {_e}")))?;

    let output = match output {
        YoutubeDlOutput::Playlist(_) => {
            return Err(ErrorInternalServerError(
                "yt-dl returned a playlist instead of a single video",
            ))
        }
        YoutubeDlOutput::SingleVideo(s) => s,
    };

    let formats = output
        .formats
        .ok_or_else(|| ErrorInternalServerError("No formats returned from yt-dl"))?;

    let video_url = formats
        .get(0)
        .ok_or_else(|| ErrorInternalServerError("empty formats array"))?
        .url
        .as_ref()
        .ok_or_else(|| ErrorInternalServerError("format doesn't have url"))?;

    let context = json! ({
        "video_title": output.title,
        "video_url": video_url,
        "tiktok_url": tiktok_url,
    });

    let rendered = tmpl
        .render("embed", &context)
        .map_err(|_e| ErrorInternalServerError("Failed to render template"))?;

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(rendered))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let mut tt = TinyTemplate::new();
        tt.add_template("embed", TEMPLATE).unwrap();
        App::new().app_data(web::Data::new(tt)).service(greet)
    })
    .bind(("0.0.0.0", 8085))?
    .run()
    .await
}
