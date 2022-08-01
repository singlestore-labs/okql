import { KqlToSql } from './kql-to-sql.js';

console.log('KTOS: Beginning');
const ktos = new KqlToSql();
const imports = {
    wasi_snapshot_preview1: {
        fd_write: function() {},
        environ_get: function() {},
        environ_sizes_get: function() {},
        proc_exit: function() {}
    }
};
ktos.instantiate(fetch('./converter_wasm.wasm'), imports)
    .then(() => {
        console.log('KTOS: Instantiated');
        const input_area = document.getElementById('input-raw');
        const output_area = document.getElementById('output-raw');

        const update = () => {
            const kql = input_area.value.replace("\r\n", "\n");
            const result = ktos.convert(kql);
            if (result.tag === 'ok') {
                output_area.value = result.val;
            } else {
                console.error(result.val);
            }
        }

        update();

        input_area.addEventListener('input', () => {
            console.log('KTOS: Input Changed');
            update();
        })
    })