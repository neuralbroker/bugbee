"use client";

import { Canvas, useFrame } from "@react-three/fiber";
import { Float, Sparkles } from "@react-three/drei";
import { Bloom, EffectComposer, Noise } from "@react-three/postprocessing";
import { useRef } from "react";
import type { Mesh } from "three";

function Crystal({ position, scale }: { position: [number, number, number]; scale: number }) {
  const ref = useRef<Mesh>(null);
  useFrame((state) => { if (ref.current) ref.current.rotation.y = state.clock.elapsedTime * 0.12; });
  return <Float speed={0.65} rotationIntensity={0.25} floatIntensity={0.55}><mesh ref={ref} position={position} scale={scale}><icosahedronGeometry args={[1, 1]} /><meshStandardMaterial color="#8b7bff" emissive="#34268a" emissiveIntensity={0.75} roughness={0.3} metalness={0.82} /></mesh></Float>;
}

function Rings() {
  const ref = useRef<Mesh>(null);
  useFrame((state) => { if (ref.current) ref.current.rotation.z = state.clock.elapsedTime * 0.07; });
  return <mesh ref={ref} rotation={[1.22, 0.15, 0]}><torusGeometry args={[2.85, 0.012, 10, 100]} /><meshBasicMaterial color="#7279ff" transparent opacity={0.48} /></mesh>;
}

export function FloatingObject() {
  return <div className="hero-canvas" aria-hidden="true"><Canvas dpr={[1, 1.5]} camera={{ position: [0, 0, 7.4], fov: 45 }} gl={{ antialias: false, powerPreference: "high-performance" }}>
    <color attach="background" args={["#040506"]} />
    <ambientLight intensity={0.7} /><pointLight position={[2, 3, 3]} color="#8c7dff" intensity={15} distance={10} /><pointLight position={[-4, -2, 1]} color="#4285ff" intensity={9} distance={8} />
    <Crystal position={[-2.6, 0.65, -1.2]} scale={0.5} /><Crystal position={[2.7, -0.8, -1.7]} scale={0.35} /><Crystal position={[1.8, 1.8, -2]} scale={0.24} /><Rings /><Sparkles count={88} scale={9} size={1.25} speed={0.12} color="#aab8ff" />
    <EffectComposer multisampling={0}><Bloom intensity={0.6} luminanceThreshold={0.85} mipmapBlur /><Noise opacity={0.035} /></EffectComposer>
  </Canvas></div>;
}
