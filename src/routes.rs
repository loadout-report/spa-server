use tracing::span::Span;
use warp::Filter;
use warp::filters::trace::Info;
use warp::reject::Rejection;

pub fn build_routes(asset_dir: String, index_path: String) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
    let assets = warp::fs::dir(asset_dir);

    let index = warp::get()
        .and(warp::fs::file(index_path));

    assets
        .or(index)
        .with(warp::trace::request())
}
