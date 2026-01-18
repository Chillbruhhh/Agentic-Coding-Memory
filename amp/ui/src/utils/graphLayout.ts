import { Vector3 } from 'three';
import { AmpObject, GraphNode } from '../types/amp';

export const createHierarchicalLayout = (objects: AmpObject[]): GraphNode[] => {
  if (!objects || objects.length === 0) return [];

  // Build hierarchy map
  const nodeMap = new Map<string, GraphNode>();
  const rootNodes: GraphNode[] = [];

  // Initialize all nodes
  objects.forEach(obj => {
    const node: GraphNode = {
      ...obj,
      x: 0,
      y: 0,
      z: 0,
      children: [],
      level: 0,
      collapsed: false
    };
    nodeMap.set(obj.id, node);
  });

  // Find root nodes and organize by type
  const projectNodes = objects.filter(obj => obj.type === 'project');
  const directoryNodes = objects.filter(obj => obj.type === 'directory');
  const fileNodes = objects.filter(obj => obj.type === 'file');
  const symbolNodes = objects.filter(obj => obj.type === 'symbol');

  // Create simple hierarchical layout
  let currentX = 0;
  const spacing = 15;

  // Position project nodes
  projectNodes.forEach((obj, index) => {
    const node = nodeMap.get(obj.id)!;
    node.x = currentX;
    node.y = 0;
    node.z = 0;
    node.level = 0;
    rootNodes.push(node);
    currentX += spacing;
  });

  // Position directory nodes
  directoryNodes.forEach((obj, index) => {
    const node = nodeMap.get(obj.id)!;
    node.x = (index % 10) * spacing - 50;
    node.y = -10;
    node.z = Math.floor(index / 10) * spacing;
    node.level = 1;
  });

  // Position file nodes
  fileNodes.forEach((obj, index) => {
    const node = nodeMap.get(obj.id)!;
    node.x = (index % 15) * spacing - 75;
    node.y = -20;
    node.z = Math.floor(index / 15) * spacing;
    node.level = 2;
  });

  // Position symbol nodes
  symbolNodes.forEach((obj, index) => {
    const node = nodeMap.get(obj.id)!;
    node.x = (index % 20) * spacing - 100;
    node.y = -30;
    node.z = Math.floor(index / 20) * spacing;
    node.level = 3;
  });

  return Array.from(nodeMap.values());
};

export const getNodeColor = (type: string): string => {
  switch (type) {
    case 'project': return '#00ffff'; // Cyan
    case 'directory': return '#ff00ff'; // Magenta
    case 'file': return '#ffff00'; // Yellow
    case 'symbol': return '#00ff00'; // Green
    default: return '#ffffff'; // White
  }
};

export const getNodeSize = (type: string): number => {
  switch (type) {
    case 'project': return 2.0;
    case 'directory': return 1.5;
    case 'file': return 1.0;
    case 'symbol': return 0.5;
    default: return 1.0;
  }
};
