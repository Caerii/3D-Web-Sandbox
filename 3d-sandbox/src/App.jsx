import React, { useState, useRef, useEffect } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { AmmoPhysics } from '../js/Ammo'; // Make sure the path is correct
import './App.css';

function App() {
  const [count, setCount] = useState(0);

  const canvasRef = useRef();
  const scene = useRef();
  const camera = useRef();
  const renderer = useRef();
  const cube = useRef();
  const controls = useRef();
  const velocity = useRef(new THREE.Vector3());

  useEffect(() => {
    scene.current = new THREE.Scene();
    scene.current.background = new THREE.Color(0x222222);

    camera.current = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.current.position.z = 5;

    renderer.current = new THREE.WebGLRenderer({ canvas: canvasRef.current });
    renderer.current.setSize(window.innerWidth, window.innerHeight);

    controls.current = new OrbitControls(camera.current, renderer.current.domElement);

    // Physics setup
    const physics = new AmmoPhysics(scene.current, { gravity: new THREE.Vector3(0, -9.8, 0) });

    // Create a cube
    const cubeGeometry = new THREE.BoxGeometry(1, 1, 1);
    const cubeMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000 });
    cube.current = new THREE.Mesh(cubeGeometry, cubeMaterial);
    scene.current.add(cube.current);

    physics.addMesh(cube.current);

    // Handle window resize
    const onWindowResize = () => {
      camera.current.aspect = window.innerWidth / window.innerHeight;
      camera.current.updateProjectionMatrix();
      renderer.current.setSize(window.innerWidth, window.innerHeight);
    };

    window.addEventListener('resize', onWindowResize);

    // Handle key presses
    const moveDirection = new THREE.Vector3();
    const onKeyDown = (event) => {
      switch (event.key) {
        case 'w':
          moveDirection.z = -1;
          break;
        case 's':
          moveDirection.z = 1;
          break;
        case 'a':
          moveDirection.x = -1;
          break;
        case 'd':
          moveDirection.x = 1;
          break;
      }
      updateVelocity();
    };

    const onKeyUp = (event) => {
      switch (event.key) {
        case 'w':
        case 's':
          moveDirection.z = 0;
          break;
        case 'a':
        case 'd':
          moveDirection.x = 0;
          break;
      }
      updateVelocity();
    };

    const updateVelocity = () => {
      const speed = 2; // Adjust this for desired movement speed
      velocity.current.set(moveDirection.x, 0, moveDirection.z).normalize().multiplyScalar(speed);
    };

    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('keyup', onKeyUp);

    // Animation loop
    const animate = () => {
      const deltaTime = renderer.current.info.render.frame / 1000;

      // Update cube's position based on velocity
      cube.current.position.add(velocity.current.clone().multiplyScalar(deltaTime));

      // Render the scene
      renderer.current.render(scene.current, camera.current);

      requestAnimationFrame(animate);
    };

    animate();

    // Cleanup
    return () => {
      window.removeEventListener('resize', onWindowResize);
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('keyup', onKeyUp);
      controls.current.dispose();
      renderer.current.dispose();
      physics.dispose();
    };
  }, []);

  return (
    <div>
      <canvas ref={canvasRef}></canvas>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/App.jsx</code> and save to test HMR
        </p>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </div>
  );
}

export default App;
