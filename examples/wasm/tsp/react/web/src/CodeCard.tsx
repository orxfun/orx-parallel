type CodeCardProps = {
    title: string;
    helpTitle: string;
    helpBody: string;
    code: string;
};

export function CodeCard({ title, helpTitle, helpBody, code }: CodeCardProps) {
    return (
        <>
            <div className="code-card-header">
                <h2>{title}</h2>
                <details className="code-help">
                    <summary className="code-help-trigger" aria-label={`Show ${title.toLowerCase()} explanation`}>
                        ?
                    </summary>
                    <div className="code-help-popover" role="note">
                        <h2 style={{ paddingLeft: 10 }}>{helpTitle}</h2>
                        <pre className="code-block">
                            <code className="language-rust">{helpBody}</code>
                        </pre>
                    </div>
                </details>
            </div>
            <pre className="code-block">
                <code className="language-rust">{code}</code>
            </pre>
        </>
    );
}
