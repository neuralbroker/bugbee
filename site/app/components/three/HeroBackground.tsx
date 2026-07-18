"use client";

import { useRef, useMemo, useEffect, useState } from "react";
import { Canvas, useFrame, useThree } from "@react-three/fiber";
import * as THREE from "three";
import {
  meshGradientFragment,
  meshGradientVertex,
} from "./shaders/meshGradient";
import { useReducedMotion } from "@/app/hooks/useReducedMotion";

function GradientMesh() {
  const meshRef = useRef<THREE.Mesh>(null);
  const materialRef = useRef<THREE.ShaderMaterial>(null);
  const mouse = useRef(new THREE.Vector2(0, 0));
  const targetMouse = useRef(new THREE.Vector2(0, 0));
  const { viewport } = useThree();

  useEffect(() => {
    const onMove = (e: MouseEvent) => {
      targetMouse.current.x = (e.clientX / window.innerWidth) * 2 - 1;
      targetMouse.current.y = -(e.clientY / window.innerHeight) * 2 + 1;
    };
    window.addEventListener("mousemove", onMove, { passive: true });
    return () => window.removeEventListener("mousemove", onMove);
  }, []);

  const uniforms = useMemo(
    () => ({
      uTime: { value: 0 },
      uMouse: { value: new THREE.Vector2(0, 0) },
    }),
    []
  );

  useFrame((state) => {
    if (!materialRef.current) return;
    materialRef.current.uniforms.uTime.value = state.clock.elapsedTime;
    mouse.current.lerp(targetMouse.current, 0.05);
    materialRef.current.uniforms.uMouse.value.copy(mouse.current);
    if (meshRef.current) {
      meshRef.current.rotation.x = mouse.current.y * 0.04;
      meshRef.current.rotation.y = mouse.current.x * 0.04;
    }
  });

  return (
    <mesh ref={meshRef} scale={[viewport.width * 1.2, viewport.height * 1.2, 1]}>
      <planeGeometry args={[1, 1, 64, 64]} />
      <shaderMaterial
        ref={materialRef}
        vertexShader={meshGradientVertex}
        fragmentShader={meshGradientFragment}
        uniforms={uniforms}
      />
    </mesh>
  );
}

export function HeroBackground() {
  const reduced = useReducedMotion();
  const [mounted, setMounted] = useState(false);
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    setMounted(true);
    setIsMobile(window.innerWidth < 768);
  }, []);

  if (!mounted || reduced || isMobile) {
    return (
      <div
        className="absolute inset-0 -z-10"
        style={{
          background:
            "radial-gradient(ellipse 80% 60% at 50% 30%, rgba(139,92,246,0.25) 0%, transparent 55%), radial-gradient(ellipse 50% 40% at 70% 60%, rgba(245,158,11,0.12) 0%, transparent 50%), #050508",
        }}
        aria-hidden
      />
    );
  }

  return (
    <div className="absolute inset-0 -z-10 opacity-80" aria-hidden>
      <Canvas
        dpr={[1, 1.5]}
        camera={{ position: [0, 0, 1.2], fov: 50 }}
        gl={{ antialias: false, alpha: true, powerPreference: "high-performance" }}
        style={{ width: "100%", height: "100%" }}
      >
        <GradientMesh />
      </Canvas>
      <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-void/40 via-transparent to-void" />
    </div>
  );
}
