use itertools::Itertools;
use polars::prelude::*;
use pyo3_polars::derive::polars_expr;

#[polars_expr(output_type=Float64)]
fn pl_psi(inputs: &[Series]) -> PolarsResult<Series> {
    // The actual data
    let data = inputs[0].f64()?;
    // breaks according to reference, already sorted
    let brk = inputs[1].f64()?;
    if brk.len() < 2 {
        return Err(PolarsError::ComputeError(
            "PSI: Not enough bins can be created.".into(),
        ));
    }
    // cnts for each brk in ref
    let cnt_ref = inputs[2].u32()?;
    // Vec to do binary search with.
    let brk_vec: Vec<f64> = brk.into_no_null_iter().collect_vec();
    // Compute the correct cnt (of values that are inside the bins defined by ref)
    let mut cnt_data = vec![0_u32; brk_vec.len()];
    for d in data.into_no_null_iter() {
        // values in brk_vec is guaranteed to be sorted, unique, and finite
        let res = brk_vec.binary_search_by(|x| x.partial_cmp(&d).unwrap());
        let idx = match res {
            Ok(i) => i,
            Err(j) => j,
        };
        cnt_data[idx] += 1;
    }
    // Total cnt in ref 
    let ref_total = cnt_ref.sum().unwrap() as f64;
    // Total cnt in actual
    let act_total = cnt_data.iter().sum::<u32>() as f64;
    // PSI
    let psi = cnt_ref
        .into_no_null_iter()
        .zip(cnt_data.into_iter())
        .fold(0., |acc, (a, b)| {
            let aa = ((a as f64) / ref_total).max(0.0001_f64);
            let bb = ((b as f64) / act_total).max(0.0001_f64);
            acc + (aa - bb) * (aa / bb).ln()
        });
    let out = Float64Chunked::from_iter([Some(psi)]);
    Ok(out.into_series())
}

// #[polars_expr(output_type=Float64)]
// fn pl_psi_discrete(inputs: &[Series]) -> PolarsResult<Series> {

//     let df1 = df!(
//         "data_discrete" => &inputs[0], // data cats
//         "data_cnt" => &inputs[1], // data cnt
//     )?;
//     let df2 = df!(
//         "ref_discrete" => &inputs[2], // ref cats
//         "ref_cnt" => &inputs[3], // ref cnt
//     )?;

//     let mut df = df1.lazy().join(df2.lazy(), [col("data_discrete")], [col("ref_discrete")], JoinArgs::new(JoinType::Outer { coalesce: false })).fill_null(0).collect()?;

//     let data_total = inputs[1].sum::<u32>().unwrap() as f64;
//     let ref_total = inputs[3].sum::<u32>().unwrap() as f64;

//     let cnt_data = df.drop_in_place("data_cnt")?;
//     let cnt_data = cnt_data.u32()?;
//     let cnt_ref = df.drop_in_place("ref_cnt")?;
//     let cnt_ref = cnt_ref.u32()?;
//     // PSI
//     let psi = cnt_ref
//         .into_no_null_iter()
//         .zip(cnt_data.into_no_null_iter())
//         .fold(0., |acc, (a, b)| {
//             let aa = (a as f64).clamp(0.0001, f64::MAX) / ref_total;
//             let bb = (b as f64).clamp(0.0001, f64::MAX) / data_total;
//             acc + (aa - bb) * (aa / bb).ln()
//         });
//     let out = Float64Chunked::from_iter([Some(psi)]);
//     Ok(out.into_series())
// }