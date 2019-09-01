strokeWeight(3);

var assert_eq = function(a, b, msg) {
    if (a !== b) {
        fill(255, 0, 0);
        text(msg, 15, 30);
    }
};

var line_vec = function (p0, p1) {
    line(p0.x, p0.y, p1.x, p1.y);
};

var quad_vecs = function(points) {
    quad(points[0].x, points[0].y, points[1].x, points[1].y, points[2].x, points[2].y, points[3].x, points[3].y);
};

var new_point = function(points) {
    if (points.length === 0) {
        return new PVector(mouseX, mouseY);
    }
    if (points.length === 1) {
        return new PVector(Math.max(mouseX, points[0].x), points[0].y);
    }
    if (points.length === 2) {
        return new PVector(mouseX, Math.min(mouseY, points[0].y));
    }
    if (points.length === 3) {
        return new PVector(Math.min(mouseX, points[2].x), points[2].y);
    }
    return null;
};

var poly_4 = function(points) {
    pushStyle();
    fill(98, 96, 232);
    noFill();
    var p = new_point(points);
    if (points.length === 1) {
        stroke(220, 95, 227);
        line_vec(points[0], p);
    }
    if (points.length === 2) {
        line_vec(points[0], points[1]);
        stroke(220, 95, 227);
        line_vec(points[1], p);
    }
    if (points.length === 3) {
        line_vec(points[0], points[1]);
        line_vec(points[1], points[2]);
        stroke(220, 95, 227);
        line_vec(points[2], p);
        line_vec(p, points[0]);
    }
    if (points.length === 4) {
        noStroke();
        fill(98, 96, 232);
        quad_vecs(points);
    }
    popStyle();
};

var frustrum = [];

var draw_sections = function(points, r) {
    pushStyle();
    noStroke();
    fill(255, 232, 84);
    for (var i = 0; i < points.length; i++) {
        var p0 = points[i];
        ellipse(p0.x, p0.y, 2*r, 2*r);
    }
    
    fill(250, 130, 172);
    for (var i = 0; i < (points.length === 4 ? 4 : points.length - 1); i++) {
        var j = (i + 1) % 4;
        var p0 = points[i];
        var p1 = points[j];
        var e = PVector.sub(p1, p0);
        var n = PVector.mult(new PVector(-e.y, e.x), r / e.mag());

        quad_vecs([
            p0,
            PVector.add(p0, n),
            PVector.add(p1, n),
            p1,
        ]);
    }
    popStyle();
};

var test_intersect = function(frustrum, c, r) {
    var inside_count = 0;
    
    var p = frustrum;
    
    var e = [
        PVector.sub(p[1], p[0]),
        PVector.sub(p[2], p[1]),
        PVector.sub(p[3], p[2]),
        PVector.sub(p[0], p[3]),
    ];
    
    var n = [
        PVector.normalize(new PVector(-e[0].y, e[0].x)),
        PVector.normalize(new PVector(-e[1].y, e[1].x)),
        PVector.normalize(new PVector(-e[2].y, e[2].x)),
        PVector.normalize(new PVector(-e[3].y, e[3].x)),
    ];
    
    var q = [
        PVector.sub(c, p[0]),
        PVector.sub(c, p[1]),
        PVector.sub(c, p[2]),
        PVector.sub(c, p[3]),
    ];
    
    // Component of q along n.
    var n_t = [
        PVector.dot(q[0], n[0]),
        PVector.dot(q[1], n[1]),
        PVector.dot(q[2], n[2]),
        PVector.dot(q[3], n[3]),
    ];

    // Component of q along e.
    var e_t = [
        PVector.dot(q[0], e[0]) / PVector.dot(e[0], e[0]),
        PVector.dot(q[1], e[1]) / PVector.dot(e[1], e[1]),
        PVector.dot(q[2], e[2]) / PVector.dot(e[2], e[2]),
        PVector.dot(q[3], e[3]) / PVector.dot(e[3], e[3]),
    ];

    // Outside quadliteral with edges displaced by r*n.
    for (var i = 0; i < 4; i++) {
        if (n_t[i] > r) {
            return null;
        }
    }
    
    // Center inside quadliteral.
    var inside_count = 0;
    for (var i = 0; i < 4; i++) {
        if (n_t[i] <= 0.0) {
            inside_count += 1;
        }
    }
    if (inside_count === 4) {
        return {
            point: c,
        };
    }
    
    // Inside one of the edge-extended boxes.
    for (var i = 0; i < 4; i++) {
        if (n_t[i] >= 0.0 && n_t[i] <= r && e_t[i] >= 0.0 && e_t[i] <= 1.0) {
            return {
                edge: i,
                point: PVector.add(p[i], PVector.mult(e[i], e_t[i])),
            };
        }
    }
    
    // Inside one of the corners.
    for (var i = 0; i < 4; i++) {
        var v = PVector.sub(c, p[i]);
        var d_sq = PVector.dot(v, v);
        if (d_sq < r*r) {
            return {
                corner: i,
                point: p[i],
            };
        }
    }
    
    // In one of the corner areas but too far from the corner.
    return null;
};

draw = function() {
    background(255, 255, 255);
    
    if (frustrum.length === 0) {
        pushStyle();
        fill(0, 0, 0);
        text("Click to draw.", 15, 30);
        popStyle();
    }
    
    var c = new PVector(mouseX, mouseY);
    var r = 69;
    
    var hit = null;
    
    if (frustrum.length === 4) {
        hit = test_intersect(frustrum, c, r);
    }

    draw_sections(frustrum, r);
    poly_4(frustrum);
    
    pushStyle();
    noFill();
    if (hit !== null) {
        stroke(106, 224, 76);
    } else {
        stroke(98, 96, 232);  
    }
    ellipse(c.x, c.y, r*2.0, r*2.0);
    popStyle();
    
    if (hit !== null && hit.point !== undefined) {
        pushStyle();
        stroke(255, 0, 0);
        line_vec(hit.point, c);
        popStyle();
    }
    
    if (hit !== null && hit.edge !== undefined) {
        pushStyle();
        stroke(255, 0, 0);
        var p0 = frustrum[hit.edge];
        var p1 = frustrum[(hit.edge + 1) % 4];
        line_vec(p0, p1);
        popStyle();
    } else if (hit !== null && hit.corner !== undefined) {
        pushStyle();
        strokeWeight(7);
        stroke(255, 0, 0);
        var p = frustrum[hit.corner];
        point(p.x, p.y);
        strokeWeight(3);
        popStyle();
    }
};

mousePressed = function() {
    if (frustrum.length === 4) {
        frustrum = [];
    }
    var p = new_point(frustrum);
    if (p !== null) {
        frustrum.push(p);
    }
};
