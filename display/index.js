function mlog (...args) {
    window.electronAPI.mlog(args);
}

/**@type {SVGElement} */
const output = document.getElementById("output");

const XMLNS = "www.w3.org/2000/svg";

// const data = window.electronAPI.dataFetch();
const data = {
    width : 2,
    height : 3,
    data : [0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]
};
mlog("data fetch");

output.setAttribute("viewBox", `0 0 ${data.width} ${data.height}`);
mlog("1");

let x = 0;
let y = 0;

function fhex (r, g, b, a) {
    const conv = "0123456789abcdef";
    const fh = (n) => {
        const m = n % 16;
        return `${conv[(n-m)/16]}${conv[m]}`;
    };
    return `#${fh(r)}${fh(g)}${fh(b)}${fh(a)}`;
}

for (let i = 0; i < data.data.length; i += 4) {
    /**@type {SVGRect} */
    const r = document.createElementNS(XMLNS, "rect");
    mlog("2", x, y);
    r.x = x;
    r.y = y;
    r.width = 1;
    r.height = 1;
    r.setAttribute("fill", "red");
    // r.style.cssText = `fill:${fhex(data.data.slice(i, i+4))}`;
    mlog("3");
    output.appendChild(r);
    mlog("4");
    x += 1;
    if (x >= data.width) {
        x = 0;
        y += 1;
    }
}

mlog(output.children.length);