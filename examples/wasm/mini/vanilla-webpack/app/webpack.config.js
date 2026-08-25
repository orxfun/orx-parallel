import path from "node:path";
import { fileURLToPath } from "node:url";
import HtmlWebpackPlugin from "html-webpack-plugin";
import { orxParallelWasm } from "orx-parallel-wasm/webpack";

const projectRoot = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(projectRoot, "dist");

export default {
    entry: "./src/main.ts",
    output: {
        path: distDir,
        filename: "assets/[name].[contenthash].js",
        publicPath: "auto",
        clean: true
    },
    resolve: {
        extensions: [".ts", ".js"]
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                exclude: /node_modules/,
                use: "ts-loader"
            },
            {
                test: /\.css$/,
                use: ["style-loader", "css-loader"]
            }
        ]
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: "./index.html"
        }),
        orxParallelWasm({
            bindings: "../wasm_bindings"
        })
    ],
    devServer: {
        static: {
            directory: distDir
        },
        hot: true
    }
};
