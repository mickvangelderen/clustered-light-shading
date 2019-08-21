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
    stroke(57, 0, 214);
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
        fill(7, 0, 214);
        quad_vecs(points);
    }
    popStyle();
};

var frustrum = [];

var draw_sections = function(points, r) {
    pushStyle();
    noStroke();
    fill(255, 221, 0);
    for (var i = 0; i < points.length; i++) {
        var p0 = points[i];
        ellipse(p0.x, p0.y, 2*r, 2*r);
    }
    
    fill(255, 43, 114);
    for (var i = 0; i < (points.length === 4 ? 4 : points.length - 1); i++) {
        var j = (i + 1) % 4;
        var p0 = points[i];
        var p1 = points[j];
        var b = PVector.sub(p1, p0);
        var n = PVector.mult(new PVector(-b.y, b.x), r / b.mag());
        quad_vecs([
            p0,
            PVector.add(p0, n),
            PVector.add(p1, n),
            p1,
        ]);
    }
    popStyle();
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
    
    var hit_corner = -1;
    var hit_corner_d_sq = r*r;
    var hit_edge = -1;
    var hit = false;
    
    var hit_point = null;
    
    if (frustrum.length === 4) {
        var inside_count = 0;
        for (var i = 0; i < 4; i++) {
            var j = (i + 1) % 4;
            var p0 = frustrum[i];
            var p1 = frustrum[j];
            var b = PVector.sub(p1, p0);
            var n = PVector.div(new PVector(-b.y, b.x), b.mag());
            var a = PVector.sub(c, p0);
            var side_d = PVector.dot(a, n);
            
            if (side_d > r) {
                // outside frustrum enlarged by d.
                hit = false;
                break;
            } else if (side_d > 0.0) {
                var t = PVector.dot(a, b) / PVector.dot(b, b);
                var ci;
                if (t < 0.0) {
                    ci = i;
                } else if (t > 1.0) {
                    ci = (i + 1) % 4;
                } else {
                    hit = true;
                    hit_edge = i;
                    hit_point = PVector.add(p0, PVector.mult(b, t));
                    break;
                }

                var corner = frustrum[ci];
                var dv = PVector.sub(corner, c);
                var d_sq = PVector.dot(dv, dv);
                
                if (d_sq < hit_corner_d_sq) {
                    hit = true;
                    hit_corner = ci;
                    hit_point = corner;
                    hit_corner_d_sq = d_sq;
                }
            } else {
                // inside frustrum.
                inside_count += 1;
            }
        }
        
        if (inside_count === 4) {
            // On the inside of all edges.
            hit = true;
            hit_point = c;
        }
    }

    draw_sections(frustrum, r);
    poly_4(frustrum);
    
    pushStyle();
    noStroke();
    if (hit) {
        fill(89, 237, 52);
    } else {
        fill(7, 0, 214);
    }
    ellipse(c.x, c.y, r*2.0, r*2.0);
    popStyle();
    
    if (hit_point !== null) {
        pushStyle();
        stroke(240, 160, 0);
        line_vec(hit_point, c);
        popStyle();
    }
    
    if (hit_corner !== -1) {
        pushStyle();
        strokeWeight(7);
        stroke(255, 0, 0);
        var p = frustrum[hit_corner];
        point(p.x, p.y);
        strokeWeight(3);
        popStyle();
    }
    
    if (hit_edge !== -1) {
        pushStyle();
        stroke(255, 0, 0);
        var p0 = frustrum[hit_edge];
        var p1 = frustrum[(hit_edge + 1) % 4];
        line_vec(p0, p1);
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
