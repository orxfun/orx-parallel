import path from "node:path";
import { fileURLToPath } from "node:url";
import { rspack } from "@rspack/core";
import { orxParallelWasm } from "orx-parallel-wasm/rspack";

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
                use: [
                    {
                        loader: "builtin:swc-loader",
                        options: {
                            jsc: {
                                parser: { syntax: "typescript" },
                                target: "es2020"
                            }
                        }
                    }
                ],
                type: "javascript/auto"
            },
            {
                test: /\.css$/,
                use: ["style-loader", "css-loader"]
            }
        ]
    },
    plugins: [
        new rspack.HtmlRspackPlugin({
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
