from cvxpy.atoms.affine.vec import vec
from cvxpy.reductions.dgp2dcp import util


def quad_over_lin_canon(expr, args):
    x = vec(args[0])
    y = args[1]
    numerator = util.sum(2 * xi for xi in x)
    return numerator - y, []
