function getRotationPrecomputeL(precompute_L, rotationMatrix) {
	let matrix = mat4Matrix2mathMatrix(rotationMatrix);

	let m3 = computeSquareMatrix_3by3(matrix);
	let m5 = computeSquareMatrix_5by5(matrix);

	let result = [];
	for (let precompute_L_channel of precompute_L) {
		let c3 = getLevel(1, precompute_L_channel);
		let c5 = getLevel(2, precompute_L_channel);

		let r3 = math.multiply(c3, m3).toArray();
		let r5 = math.multiply(c5, m5).toArray();

		temp = [precompute_L_channel[0]];
		for (let r of r3) {
			temp.push(r);
		}
		for (let r of r5) {
			temp.push(r);
		}
		result.push(temp);
	}
	return result;
}

function computeSquareMatrix_3by3(rotationMatrix) { // 计算方阵SA(-1) 3*3 
	// 1、pick ni - {ni}
	let n1 = [1, 0, 0, 0]; let n2 = [0, 0, 1, 0]; let n3 = [0, 1, 0, 0];
	let n = [n1, n2, n3];
	return computeSquareMatrixHelp(1, n, rotationMatrix);
}

function computeSquareMatrix_5by5(rotationMatrix) { // 计算方阵SA(-1) 5*5
	// 1、pick ni - {ni}
	let k = 1 / math.sqrt(2);
	let n1 = [1, 0, 0, 0]; let n2 = [0, 0, 1, 0]; let n3 = [k, k, 0, 0];
	let n4 = [k, 0, k, 0]; let n5 = [0, k, k, 0];
	let n = [n1, n2, n3, n4, n5];
	return computeSquareMatrixHelp(2, n, rotationMatrix);
}

function getLevel(l, p) {
	let temp = [];
	for (let i = 0; i < 2 * l + 1; i++) {
		temp.push(p[l * l + i]);
	}
	return temp;
}

function computeSquareMatrixHelp(l, n, rotationMatrix) {
	// 2、{P(ni)} - A  A_inverse
	let pn = [];
	for (let ni of n) {
		let p = SHEval(ni[0], ni[1], ni[2], 3);
		pn.push(getLevel(l, p));
	}
	let A = math.matrix(pn);
	let A_inverse = math.inv(A);

	// 3、用 R 旋转 ni - {R(ni)}
	let r = [];
	for (let ni of n) {
		let ri = math.multiply(ni, rotationMatrix);
		r.push(ri.toArray());
	}

	// 4、R(ni) SH投影 - S
	let s = [];
	for (let ri of r) {
		let si = SHEval(ri[0], ri[1], ri[2], 3);
		s.push(getLevel(l, si));
	}
	let S = math.matrix(s);

	// 5、S*A_inverse
	return math.multiply(S, A_inverse);
}

function mat4Matrix2mathMatrix(rotationMatrix) {
	// ! mat4是行优先， math.js 是列优先
	let mathMatrix = [];
	for (let i = 0; i < 4; i++) {
		let r = [];
		for (let j = 0; j < 4; j++) {
			r.push(rotationMatrix[j * 4 + i]);
		}
		mathMatrix.push(r);
	}
	return math.matrix(mathMatrix)

}

function getMat3ValueFromRGB(precomputeL) {

	let colorMat3 = [];
	for (var i = 0; i < 3; i++) {
		colorMat3[i] = mat3.fromValues(precomputeL[0][i], precomputeL[1][i], precomputeL[2][i],
			precomputeL[3][i], precomputeL[4][i], precomputeL[5][i],
			precomputeL[6][i], precomputeL[7][i], precomputeL[8][i]);
	}
	return colorMat3;
}