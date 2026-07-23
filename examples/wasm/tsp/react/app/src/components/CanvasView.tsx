import { useEffect, useRef } from "react";
import type { Location, SearchResult } from "../shared-types";

type CanvasViewProps = {
    points: Location[];
    best: SearchResult | null;
    cityNodeColor: string;
    tourLineColor: string;
    canvasBackgroundColor: string;
};

export function CanvasView(props: CanvasViewProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) {
            return;
        }

        const ctx = canvas.getContext("2d");
        if (!ctx) {
            return;
        }

        drawPoints(ctx, canvas, props.points, props.cityNodeColor, props.canvasBackgroundColor);

        if (props.best && props.best.best_tour.length > 0) {
            drawTour(ctx, canvas, props.points, props.best.best_tour, props.cityNodeColor, props.tourLineColor, props.canvasBackgroundColor);
        }
    }, [props.points, props.best, props.cityNodeColor, props.tourLineColor, props.canvasBackgroundColor]);

    return <canvas id="canvas" ref={canvasRef} width={920} height={430}></canvas>;
}

function drawPoints(
    ctx: CanvasRenderingContext2D,
    canvas: HTMLCanvasElement,
    locations: Location[],
    cityNodeColor: string,
    canvasBackgroundColor: string
) {
    ctx.fillStyle = canvasBackgroundColor;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const mapped = mapPoints(locations, canvas.width, canvas.height);
    for (const point of mapped) {
        ctx.beginPath();
        ctx.fillStyle = cityNodeColor;
        ctx.arc(point.x, point.y, 6, 0, Math.PI * 2);
        ctx.fill();
    }
}

function drawTour(
    ctx: CanvasRenderingContext2D,
    canvas: HTMLCanvasElement,
    locations: Location[],
    tour: number[],
    cityNodeColor: string,
    tourLineColor: string,
    canvasBackgroundColor: string
) {
    drawPoints(ctx, canvas, locations, cityNodeColor, canvasBackgroundColor);

    const mapped = mapPoints(locations, canvas.width, canvas.height);

    ctx.beginPath();
    ctx.strokeStyle = tourLineColor;
    ctx.lineWidth = 2;

    const first = mapped[tour[0]];
    ctx.moveTo(first.x, first.y);

    for (const index of tour.slice(1)) {
        const point = mapped[index];
        ctx.lineTo(point.x, point.y);
    }

    ctx.lineTo(first.x, first.y);
    ctx.stroke();
}

function mapPoints(locations: Location[], width: number, height: number) {
    const pad = 28;
    const xs = locations.map((p) => p.x);
    const ys = locations.map((p) => p.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    const spanX = Math.max(maxX - minX, 1);
    const spanY = Math.max(maxY - minY, 1);

    return locations.map((p) => ({
        x: pad + ((p.x - minX) / spanX) * (width - pad * 2),
        y: height - (pad + ((p.y - minY) / spanY) * (height - pad * 2))
    }));
}
