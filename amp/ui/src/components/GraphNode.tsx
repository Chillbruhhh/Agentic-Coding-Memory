import React, { useRef, useState } from 'react';
import { useFrame } from '@react-three/fiber';
import { Text } from '@react-three/drei';
import { Mesh } from 'three';
import { GraphNode as GraphNodeType } from '../types/amp';
import { getNodeColor, getNodeSize } from '../utils/graphLayout';

interface GraphNodeProps {
  node: GraphNodeType;
  onSelect: (node: GraphNodeType) => void;
  selected: boolean;
}

export const GraphNode: React.FC<GraphNodeProps> = ({ node, onSelect, selected }) => {
  const meshRef = useRef<Mesh>(null);
  const [hovered, setHovered] = useState(false);
  
  const color = getNodeColor(node.type);
  const size = getNodeSize(node.type);
  const scale = selected ? 1.5 : hovered ? 1.2 : 1.0;

  useFrame((state) => {
    if (meshRef.current) {
      // Gentle floating animation
      meshRef.current.rotation.y += 0.01;
      meshRef.current.position.y = node.y + Math.sin(state.clock.elapsedTime + node.x) * 0.1;
    }
  });

  const handleClick = () => {
    onSelect(node);
  };

  return (
    <group position={[node.x, node.y, node.z]}>
      <mesh
        ref={meshRef}
        onClick={handleClick}
        onPointerOver={() => setHovered(true)}
        onPointerOut={() => setHovered(false)}
        scale={scale}
      >
        <sphereGeometry args={[size, 16, 16]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={selected ? 0.5 : hovered ? 0.3 : 0.1}
          transparent
          opacity={0.8}
        />
      </mesh>
      
      {(hovered || selected) && (
        <Text
          position={[0, size + 1, 0]}
          fontSize={0.5}
          color={color}
          anchorX="center"
          anchorY="middle"
        >
          {node.name}
        </Text>
      )}
      
      {/* Cyberpunk glow effect */}
      <mesh scale={scale * 1.1}>
        <sphereGeometry args={[size, 16, 16]} />
        <meshBasicMaterial
          color={color}
          transparent
          opacity={selected ? 0.3 : hovered ? 0.2 : 0.1}
        />
      </mesh>
    </group>
  );
};
