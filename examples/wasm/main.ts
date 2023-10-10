import { Ma } from "./pkg/examples.js";

const ma = Ma.new(0.1);

for (let i = 0; i < 10; i++) {
    console.log(ma.next(i));
}
