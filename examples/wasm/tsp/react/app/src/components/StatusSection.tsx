type StatusSectionProps = {
    isRunning: boolean;
    runSubtitle: string;
    runElapsed: string;
    bestDistance: string;
    elapsed: string;
    ips: string;
};

export function StatusSection(props: StatusSectionProps) {
    return (
        <>
            <div id="runOverlay" className={`run-overlay ${props.isRunning ? "active" : ""}`} aria-live="polite" aria-hidden={!props.isRunning}>
                <div className="run-overlay-card">
                    <div className="run-overlay-top">
                        <span className="spinner" aria-hidden="true"></span>
                        <p id="runTitle" className="run-title">
                            Running search...
                        </p>
                    </div>
                    <p id="runSubtitle" className="run-subtitle">
                        {props.runSubtitle}
                    </p>
                    <p id="runElapsed" className="run-elapsed">
                        {props.runElapsed}
                    </p>
                    <div className="run-bar" aria-hidden="true"></div>
                </div>
            </div>

            <div className="stats">
                <div className="stat">
                    <h3>Best Distance</h3>
                    <p id="bestDistance">{props.bestDistance}</p>
                </div>
                <div className="stat">
                    <h3>Elapsed</h3>
                    <p id="elapsed">{props.elapsed}</p>
                </div>
                <div className="stat">
                    <h3>Iterations/s</h3>
                    <p id="ips">{props.ips}</p>
                </div>
            </div>
        </>
    );
}
