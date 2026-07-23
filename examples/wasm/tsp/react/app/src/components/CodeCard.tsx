import { useEffect, useRef } from "react";
import hljs from "highlight.js/lib/core";

type CodeCardProps = {
    title: string;
    helpTitle: string;
    helpBody: string;
    code: string;
};

export function CodeCard({ title, helpTitle, helpBody, code }: CodeCardProps) {
    const codeRef = useRef<HTMLElement>(null);
    const helpCodeRef = useRef<HTMLElement>(null);

    useEffect(() => {
        if (codeRef.current) {
            hljs.highlightElement(codeRef.current);
        }

        if (helpCodeRef.current) {
            hljs.highlightElement(helpCodeRef.current);
        }
    }, []);

    return (
        <>
            <div className="code-card-header">
                <h2>{title}</h2>
                <details className="code-help">
                    <summary className="code-help-trigger" aria-label={`Show ${title} explanation`}>
                        ?
                    </summary>
                    <div className="code-help-popover" role="note">
                        <h2 style={{ paddingLeft: 10 }}>{helpTitle}</h2>
                        <pre className="code-block">
                            <code ref={helpCodeRef} className="language-rust">
                                {helpBody}
                            </code>
                        </pre>
                    </div>
                </details>
            </div>
            <pre className="code-block">
                <code ref={codeRef} className="language-rust">
                    {code}
                </code>
            </pre>
        </>
    );
}
