import { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import PanZoomControls from './PanZoomControls';

const STARTING_CAMERA_RADIUS = 5;
const VERTICES_PER_HEXIGON = 12;

export function Scene({gridItems, worldWidth}: {gridItems: boolean[][], worldWidth: number}) {
    const refContainer = useRef(null as HTMLDivElement | null);
    const [gridGeometry, setGridGeometry] = useState(null as THREE.BufferGeometry | null );

    useEffect(() => {
        if (!worldWidth) {
            return;
        }

        const scene = new THREE.Scene();
        const camera = new THREE.OrthographicCamera(-STARTING_CAMERA_RADIUS, STARTING_CAMERA_RADIUS, -STARTING_CAMERA_RADIUS, STARTING_CAMERA_RADIUS);
        camera.position.set(0, 0, 1);
        camera.rotation.set(0, 0, 0);
    
        const renderer = new THREE.WebGLRenderer();
        renderer.setSize(window.innerWidth, window.innerHeight);
    
        if (refContainer.current) {
            refContainer.current.appendChild(renderer.domElement);
        }

        const gridMesh = buildGridMesh(worldWidth);
        setGridGeometry(gridMesh.geometry);
        scene.add(gridMesh);
        
        new PanZoomControls(camera, renderer.domElement);
    
        function animate() {
            requestAnimationFrame(animate);
            
            renderer.render(scene, camera);
        }
        animate();

        const currentRefContainer = refContainer.current;
        return () => currentRefContainer?.replaceChildren();
    }, [worldWidth]);

    useEffect(() => {
        if (!gridGeometry) {
            return;
        }

        gridGeometry.setAttribute('color', new THREE.Float32BufferAttribute(
            new Array(worldWidth).fill(null).flatMap((_, x) => 
                new Array(worldWidth).fill(null).flatMap((_, y) =>
                    new Array(VERTICES_PER_HEXIGON).fill(null).flatMap(() => gridItems[x][y]
                        ? [0.1, 0.1, 0.1]
                        : [1, 1, 1]
                    )
                )
            ),
            3 // r, g, b
        ));
    }, [gridItems, worldWidth, gridGeometry]);

    return <div ref={refContainer}></div>;
}

function buildGridMesh(gridSize: number) {
    const geometry = new THREE.BufferGeometry();

    const positions: number[] = [];
    
    const deltaY = Math.sqrt(3) / 2;
    for (let x = 0; x < gridSize; x++) {
        for (let y = 0; y < gridSize; y++) {
            const xCenter = (3 * x) + (3 * y / 2);
            const yCenter = deltaY * y;
            const center = new THREE.Vector3(xCenter, yCenter, 0);
            
            positions.push(
                ...getHexVertexPos(center, 0), ...getHexVertexPos(center, 1), ...getHexVertexPos(center, 5),
                ...getHexVertexPos(center, 1), ...getHexVertexPos(center, 2), ...getHexVertexPos(center, 5),
                ...getHexVertexPos(center, 2), ...getHexVertexPos(center, 4), ...getHexVertexPos(center, 5),
                ...getHexVertexPos(center, 2), ...getHexVertexPos(center, 3), ...getHexVertexPos(center, 4),
            )
        }
    }

    geometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3 /* x, y, z */));

    geometry.setAttribute('color', new THREE.Float32BufferAttribute(
        new Array(gridSize * gridSize * 12).fill(null).flatMap((_, i) => [1, 1, 1]),
        3 // r, g, b
    ))

    const material = new THREE.MeshBasicMaterial({ vertexColors: true });

    return new THREE.Mesh(geometry, material);
}
    
const angle = Math.PI / 3;
const axisVector = new THREE.Vector3(0, 0, -1);
const sideVector = new THREE.Vector3(1, 0, 0);
function getHexVertexPos(center: THREE.Vector3, vertexIndex: number) {
    return new THREE.Vector3()
        .copy(sideVector)
        .applyAxisAngle(axisVector, vertexIndex * angle).add(center);
}