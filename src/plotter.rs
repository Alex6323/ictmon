//use futures::Future;
use plotlib::line::Style as LineStyle;
use plotlib::page::Page;
use plotlib::style::Line;
use resvg::usvg;
use std::path::Path;

//type PlotFuture = Box<Future<Item = (), Error = ()>>;

pub fn render_graph(data1: &[(f64, f64)], data2: &[(f64, f64)]) -> Result<(), ()> /* -> PlotFuture */
{
    let line1 = plotlib::line::Line::new(&data1).style(LineStyle::new().colour("#DD3355"));
    let line2 = plotlib::line::Line::new(&data2).style(LineStyle::new().colour("#35C788"));

    let view = plotlib::view::ContinuousView::new()
        .add(&line1)
        .add(&line2)
        .x_label("Time")
        .y_label("TPS");

    let svg = Page::single(&view).to_svg().unwrap().to_string();

    let backend = resvg::default_backend();
    let opt = resvg::Options::default();

    let tree = match usvg::Tree::from_str(&svg, &opt.usvg) {
        Ok(t) => t,
        Err(_) => return Err(()), //Box::new(future::err(())),
    };

    let image = match backend.render_to_image(&tree, &opt) {
        Some(img) => img,
        None => return Err(()), //Box::new(future::err(())),
    };

    image.save(Path::new("graph.png"));

    //Box::new(future::ok(()))
    Ok(())
}
