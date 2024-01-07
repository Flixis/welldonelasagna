import { useEffect, useRef } from 'react';
import Chart from 'chart.js/auto';
import { ChartData, ChartOptions } from 'chart.js';
  

interface MyChartProps {
    data: ChartData;
    options: ChartOptions;
}

const Barchart = ({ data, options }: MyChartProps) => {
    const chartRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        if (chartRef.current) {
            const ctx = chartRef.current.getContext('2d');
            if (ctx) {
                const chart = new Chart(ctx, {
                    type: 'bar',
                    data: data,
                    options: options
                });
    
                // Wrap the cleanup logic in a function that returns void
                return () => {
                    chart.destroy();
                };
            }
        }
    }, [data, options]);

    return <canvas ref={chartRef} />;
};

export default Barchart;
