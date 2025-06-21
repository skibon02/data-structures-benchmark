pub fn linear_regression(data: &[(f64, f64, f64, f64)]) -> (f64, f64, f64) {
    // u ≈ a·x + b·y + c·z
    let n = data.len() as f64;

    // Суммы
    let (mut sum_x, mut sum_y, mut sum_z, mut sum_u) = (0.0, 0.0, 0.0, 0.0);
    let (mut sum_xx, mut sum_yy, mut sum_zz) = (0.0, 0.0, 0.0);
    let (mut sum_xy, mut sum_xz, mut sum_yz) = (0.0, 0.0, 0.0);
    let (mut sum_ux, mut sum_uy, mut sum_uz) = (0.0, 0.0, 0.0);

    for &(x, y, z, u) in data {
        sum_x += x;
        sum_y += y;
        sum_z += z;
        sum_u += u;

        sum_xx += x * x;
        sum_yy += y * y;
        sum_zz += z * z;

        sum_xy += x * y;
        sum_xz += x * z;
        sum_yz += y * z;

        sum_ux += u * x;
        sum_uy += u * y;
        sum_uz += u * z;
    }

    // Составляем матрицу X^T·X и X^T·y
    let xtx = [
        [sum_xx, sum_xy, sum_xz],
        [sum_xy, sum_yy, sum_yz],
        [sum_xz, sum_yz, sum_zz],
    ];
    let xty = [sum_ux, sum_uy, sum_uz];

    // Решаем линейную систему: X^T·X·β = X^T·y
    solve_3x3(xtx, xty)
}

pub fn regression_error(
    data: &[(f64, f64, f64, f64)],
    (a, b, c): (f64, f64, f64),
) -> (f64, f64, f64) {
    let n = data.len() as f64;

    // Сначала собираем матрицу X^T·X
    let mut xtx = [[0.0; 3]; 3];
    let mut residuals_squared_sum = 0.0;

    for &(x, y, z, actual) in data {
        let predicted = a * x + b * y + c * z;
        let residual = actual - predicted;
        residuals_squared_sum += residual * residual;

        let v = [x, y, z];
        for i in 0..3 {
            for j in 0..3 {
                xtx[i][j] += v[i] * v[j];
            }
        }
    }

    // sigma^2 = sum(residual^2) / (n - p), где p — число параметров (3)
    let sigma2 = residuals_squared_sum / (n - 3.0);

    // Инвертируем X^T X
    let inv = invert_3x3(xtx);
    if inv.is_none() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let inv_xtx = inv.unwrap();

    // Дисперсия коэффициентов = sigma^2 * diag((X^T X)^-1)
    let var_a = sigma2 * inv_xtx[0][0];
    let var_b = sigma2 * inv_xtx[1][1];
    let var_c = sigma2 * inv_xtx[2][2];

    // Возвращаем стандартное отклонение (sqrt дисперсии)
    (var_a.sqrt(), var_b.sqrt(), var_c.sqrt())
}

fn invert_3x3(m: [[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
    let det = |a: [[f64; 3]; 3]| -> f64 {
        a[0][0]*(a[1][1]*a[2][2] - a[1][2]*a[2][1])
            - a[0][1]*(a[1][0]*a[2][2] - a[1][2]*a[2][0])
            + a[0][2]*(a[1][0]*a[2][1] - a[1][1]*a[2][0])
    };

    let d = det(m);
    if d.abs() < 1e-12 {
        return None;
    }

    let mut inv = [[0.0; 3]; 3];
    inv[0][0] =  (m[1][1]*m[2][2] - m[1][2]*m[2][1]) / d;
    inv[0][1] = -(m[0][1]*m[2][2] - m[0][2]*m[2][1]) / d;
    inv[0][2] =  (m[0][1]*m[1][2] - m[0][2]*m[1][1]) / d;

    inv[1][0] = -(m[1][0]*m[2][2] - m[1][2]*m[2][0]) / d;
    inv[1][1] =  (m[0][0]*m[2][2] - m[0][2]*m[2][0]) / d;
    inv[1][2] = -(m[0][0]*m[1][2] - m[0][2]*m[1][0]) / d;

    inv[2][0] =  (m[1][0]*m[2][1] - m[1][1]*m[2][0]) / d;
    inv[2][1] = -(m[0][0]*m[2][1] - m[0][1]*m[2][0]) / d;
    inv[2][2] =  (m[0][0]*m[1][1] - m[0][1]*m[1][0]) / d;

    Some(inv)
}


// Решает систему 3x3 методом Крамера
fn solve_3x3(m: [[f64; 3]; 3], y: [f64; 3]) -> (f64, f64, f64) {
    let det = |a: [[f64; 3]; 3]| -> f64 {
        a[0][0] * (a[1][1]*a[2][2] - a[1][2]*a[2][1])
            - a[0][1] * (a[1][0]*a[2][2] - a[1][2]*a[2][0])
            + a[0][2] * (a[1][0]*a[2][1] - a[1][1]*a[2][0])
    };

    let d = det(m);

    if d.abs() < 1e-10 {
        panic!("Singular matrix");
    }

    let replace = |col: usize| -> [[f64; 3]; 3] {
        let mut r = m;
        for i in 0..3 {
            r[i][col] = y[i];
        }
        r
    };

    let dx = det(replace(0));
    let dy = det(replace(1));
    let dz = det(replace(2));

    (dx / d, dy / d, dz / d)
}
