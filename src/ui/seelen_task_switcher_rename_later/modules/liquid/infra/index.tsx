import * as d3 from 'd3';
import { useEffect, useRef } from 'react';

import { Placement } from '../../shared/store/domain';

import { mocked_desktops } from '../mock';

const data = {
  nodes: mocked_desktops.map((d) => d.asData()),
  links: mocked_desktops.map((d) => d.getLinks()).flat(),
};

const MINI_DESKTOP_WIDTH = 160;
const MINI_DESKTOP_HEIGHT = 90;

const chart = function () {
  // Specify the dimensions of the chart.
  const width = 928;
  const height = 600;

  // Specify the color scale.
  // const color = d3.scaleOrdinal(d3.schemeCategory10);

  // The force simulation mutates links and nodes, so create a copy
  // so that re-evaluating this cell produces the same result.
  const links = data.links.map((d) => ({ ...d }));
  const nodes = data.nodes.map((d) => ({ ...d }));

  // Create a simulation with several forces.
  const simulation = d3
    .forceSimulation(nodes)
    .force(
      'link',
      d3
        .forceLink(links)
        .id((d: any) => d.id)
        .distance(200)
        .strength(1),
    )
    .force('collide', d3.forceCollide())
    .force('charge', d3.forceManyBody())
    .force('center', d3.forceCenter(width / 2, height / 2))
    .on('tick', ticked);

  // Create the SVG container.
  const svg = d3
    .create('svg')
    .attr('width', width)
    .attr('height', height)
    .attr('viewBox', [0, 0, width, height])
    .attr('style', 'max-width: 100%; height: auto;');

  // Add a line for each link, and a circle for each node.
  const link = svg
    .append('g')
    .attr('stroke', '#999')
    .attr('stroke-opacity', 0.6)
    .selectAll()
    .data(links)
    .join('line')
    .attr('stroke-width', (d: any) => Math.sqrt(d.value));

  const node = svg
    .append('g')
    .attr('stroke', '#fff')
    .attr('stroke-width', 1.5)
    .selectAll()
    .data(nodes)
    .join('rect')
    .attr('width', MINI_DESKTOP_WIDTH)
    .attr('height', MINI_DESKTOP_HEIGHT)
    .attr('class', 'mini-desktop');

  node.append('title').text((d) => d.id);

  // Add a drag behavior.
  node.call(d3.drag().on('start', dragstarted).on('drag', dragged).on('end', dragended) as any);

  // Set the position attributes of links and nodes each time the simulation ticks.
  function ticked() {
    link
      .attr('x1', (d: any) => {
        let x = d.source.x;
        if ([Placement.Right, Placement.TopRight, Placement.BottomRight].includes(d.placement)) {
          return x + MINI_DESKTOP_WIDTH;
        }
        if ([Placement.Top, Placement.Bottom].includes(d.placement)) {
          return x + MINI_DESKTOP_WIDTH / 2;
        }
        return x;
      })
      .attr('y1', (d: any) => {
        let y = d.source.y;
        if ([Placement.Bottom, Placement.BottomLeft, Placement.BottomRight].includes(d.placement)) {
          return y + MINI_DESKTOP_HEIGHT;
        }
        if ([Placement.Left, Placement.Right].includes(d.placement)) {
          return y + MINI_DESKTOP_HEIGHT / 2;
        }
        return y;
      })
      .attr('x2', (d: any) => {
        let x = d.target.x;
        if ([Placement.Left, Placement.TopLeft, Placement.BottomLeft].includes(d.placement)) {
          return x + MINI_DESKTOP_WIDTH;
        }
        if ([Placement.Top, Placement.Bottom].includes(d.placement)) {
          return x + MINI_DESKTOP_WIDTH / 2;
        }
        return x;
      })
      .attr('y2', (d: any) => {
        let y = d.target.y;
        if ([Placement.Top, Placement.TopLeft, Placement.TopRight].includes(d.placement)) {
          return y + MINI_DESKTOP_HEIGHT;
        }
        if ([Placement.Left, Placement.Right].includes(d.placement)) {
          return y + MINI_DESKTOP_HEIGHT / 2;
        }
        return y;
      });

    (node as any).attr('x', (d: any) => d.x).attr('y', (d: any) => d.y);
  }

  // Reheat the simulation when drag starts, and fix the subject position.
  function dragstarted(event: any) {
    if (!event.active) simulation.alphaTarget(0.3).restart();
    event.subject.fx = event.subject.x;
    event.subject.fy = event.subject.y;
  }

  // Update the subject (dragged node) position during drag.
  function dragged(event: any) {
    event.subject.fx = event.x;
    event.subject.fy = event.y;
  }

  // Restore the target alpha so the simulation cools after dragging ends.
  // Unfix the subject position now that it’s no longer being dragged.
  function dragended(event: any) {
    if (!event.active) simulation.alphaTarget(0);
    event.subject.fx = null;
    event.subject.fy = null;
  }

  // When this cell is re-run, stop the previous simulation. (This doesn’t
  // really matter since the target alpha is zero and the simulation will
  // stop naturally, but it’s a good practice.)
  // invalidation.then(() => simulation.stop());

  return svg.node();
};

export function Liquid() {
  const container = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const svg = chart();
    if (svg && container.current) {
      container.current.appendChild(svg);
    }
  }, []);

  return <div className="liquid" ref={container} />;
}
