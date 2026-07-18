"use client";

import { useMemo, useRef, useEffect, useState } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { useReducedMotion } from "@/app/hooks/useReducedMotion";

const COUNT = 150;

function Particles() {
  const pointsRef = useRef<THREE.Points>(null);
  const mouse = useRef(new THREE.Vector2(0, 0));
  const target = useRef(new THREE.Vector2(0, 0));

  const { positions, colors, velocities } = useMemo(() => {
    const positions = new Float32Array(COUNT * 3);
    const colors = new Float32Array(COUNT * 3);
    const velocities = new Float32Array(COUNT * 3);
    const amber = new THREE.Color("#f59e0b");
    const purple = new THREE.Color("#8b5cf6");
    const cyan = new THREE.Color("#22d3ee");
    const palette = [amber, purple, cyan];

    for (let i = 0; i < COUNT; i++) {
      positions[i * 3] = (Math.random() - 0.5) * 12;
      positions[i * 3 + 1] = (Math.random() - 0.5) * 6;
      positions[i * 3 + 2] = (Math.random() - 0.5) * 4;
      velocities[i * 3] = (Math.random() - 0.5) * 0.004;
      velocities[i * 3 + 1] = (Math.random() - 0.5) * 0.004;
      velocities[i * 3 + 2] = (Math.random() - 0.5) * 0.002;
      const c = palette[i % 3];
      colors[i * 3] = c.r;
      colors[i * 3 + 1] = c.g;
      colors[i * 3 + 2] = c.b;
    }
    return { positions, colors, velocities };
  }, []);

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      target.current.x = (e.clientX / window.innerWidth) * 2 - 1;
      target.current.y = -(e.clientY / window.innerHeight) * 2 + 1;
    };
    window.addEventListener("mousemove", onMove, { passive: true });
    return () => window.removeEventListener("mousemove", onMove);
  }, []);

  useFrame(() => {
    if (!pointsRef.current) return;
    mouse.current.lerp(target.current, 0.08);
    const pos = pointsRef.current.geometry.attributes.position
      .array as Float32Array;
    const mx = mouse.current.x * 5;
    const my = mouse.current.y * 3;

    for (let i = 0; i < COUNT; i++) {
      const ix = i * 3;
      pos[ix] += velocities[ix];
      pos[ix + 1] += velocities[ix + 1];
      pos[ix + 2] += velocities[ix + 2];

      // Bounds wrap
      if (Math.abs(pos[ix]) > 6) velocities[ix] *= -1;
      if (Math.abs(pos[ix + 1]) > 3) velocities[ix + 1] *= -1;

      // Mouse repulsion
      const dx = pos[ix] - mx;
      const dy = pos[ix + 1] - my;
      const dist = Math.sqrt(dx * dx + dy * dy) + 0.001;
      if (dist < 1.5) {
        const force = (1.5 - dist) * 0.02;
        pos[ix] += (dx / dist) * force;
        pos[ix + 1] += (dy / dist) * force;
      }
    }
    pointsRef.current.geometry.attributes.position.needsUpdate = true;
  });

  return (
    <points ref={pointsRef}>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          args={[positions, 3]}
        />
        <bufferAttribute attach="attributes-color" args={[colors, 3]} />
      </bufferGeometry>
      <pointsMaterial
        size={0.045}
        vertexColors
        transparent
        opacity={0.7}
        sizeAttenuation
        depthWrite={false}
        blending={THREE.AdditiveBlending}
      />
    </points>
  );
}

export function ParticleField() {
  const reduced = useReducedMotion();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted || reduced) {
    return (
      <div
        className="absolute inset-0 -z-10 opacity-40"
        style={{
          background:
            "radial-gradient(ellipse at 30% 50%, rgba(245,158,11,0.15), transparent 50%), radial-gradient(ellipse at 70% 40%, rgba(139,92,246,0.12), transparent 45%)",
        }}
        aria-hidden
      />
    );
  }

  return (
    <div className="absolute inset-0 -z-10" aria-hidden>
      <Canvas
        dpr={[1, 1.5]}
        camera={{ position: [0, 0, 5], fov: 50 }}
        gl={{ antialias: false, alpha: true, powerPreference: "high-performance" }}
      >
        <Particles />
      </Canvas>
    </div>
  );
}
