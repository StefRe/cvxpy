use faer::sparse::SparseColMat;

use crate::faer_ext;
use crate::linop::CvxpyShape;
use crate::linop::Linop;
use crate::linop::LinopKind;
use crate::view::Tensor;
use crate::view::View;

pub(crate) const CONST_ID: i64 = -1;

fn get_variable_tensor(shape: &CvxpyShape, id: i64) -> Tensor {
    assert!(id > CONST_ID);
    let n = shape.numel();
    return [(id, [(CONST_ID, faer_ext::eye(n))].into())].into();
}

pub(crate) fn process_constraints<'a>(linop: &Linop, view: View<'a>) -> View<'a> {
    match linop.kind {
        LinopKind::Variable(id) => View {
            variables: [id].into(),
            tensor: get_variable_tensor(&linop.shape, id),
            is_parameter_free: true,
            context: view.context,
        },
        LinopKind::ScalarConst(c) => {
            let mat = SparseColMat::try_new_from_triplets(1, 1, &[(0, 0, c)]).unwrap();
            let tensor = [(CONST_ID, [(CONST_ID, mat)].into())].into();
            View {
                variables: [CONST_ID].into(),
                tensor,
                is_parameter_free: true,
                context: view.context,
            }
        }
        LinopKind::DenseConst(mat) => {
            let mut triplets = Vec::with_capacity(mat.ncols() * mat.nrows());
            for ((i, j), v) in mat.indexed_iter() {
                triplets.push((i as u64 + j as u64 * mat.nrows() as u64, 0_u64, *v));
            }
            let mat = SparseColMat::try_new_from_triplets(mat.nrows() * mat.ncols(), 1, &triplets)
                .unwrap();
            let tensor = [(CONST_ID, [(CONST_ID, mat)].into())].into();
            View {
                variables: [CONST_ID].into(),
                tensor,
                is_parameter_free: true,
                context: view.context,
            }
        }
        // LinopKind::SparseConst(mat) => {
        //     let tensor = [(CONST_ID, [(CONST_ID, mat)].into())].into();
        //     View {
        //         variables: [CONST_ID].into(),
        //         tensor,
        //         is_parameter_free: true,
        //         context: view.context,
        //     }
        // },
        LinopKind::Neg => neg(linop, view),
        LinopKind::Transpose => transpose(linop, view),
        LinopKind::Sum => view, // Sum (along axis 1) is implicit in Ax+b, so it is a NOOP.
        LinopKind::Reshape => view, // Reshape is a NOOP.
        LinopKind::Promote => promote(linop, view),
        _ => panic!(),
    }
}

pub(crate) fn neg<'a>(_linop: &Linop, mut view: View<'a>) -> View<'a> {
    // Second argument is not used for neg
    view.apply_all(|x, _p| -x);
    return view;
}

pub(crate) fn transpose<'a>(linop: &Linop, mut view: View<'a>) -> View<'a> {
    let rows = get_transpose_rows(&linop.shape);
    view.select_rows(&rows);
    view
}

pub(crate) fn get_transpose_rows<'a>(shape: &CvxpyShape) -> Vec<u64> {
    let (m, n) = shape.broadcasted_shape();
    let rows: Vec<u64> = (0..n)
        .flat_map(|j| (0..m).map(move |i| i * n + j))
        .collect();
    rows
}

pub(crate) fn promote<'a>(linop: &Linop, mut view: View<'a>) -> View<'a> {
    let rows = vec![0; linop.shape.numel() as usize];
    view.select_rows(&rows);
    view
}

#[cfg(test)]
mod test_backend {
    use super::*;

    #[test]
    fn test_get_transpose_rows() {
        let shape = CvxpyShape::D2(3, 2);
        let rows = get_transpose_rows(&shape);
        assert_eq!(rows, vec![0, 2, 4, 1, 3, 5]);

        let shape = CvxpyShape::D2(2, 3);
        let rows = get_transpose_rows(&shape);
        assert_eq!(rows, vec![0, 3, 1, 4, 2, 5]);

        let shape = CvxpyShape::D1(3);
        let rows = get_transpose_rows(&shape);
        assert_eq!(rows, vec![0, 1, 2]);

        let shape = CvxpyShape::D0;
        let rows = get_transpose_rows(&shape);
        assert_eq!(rows, vec![0]);
    }
}