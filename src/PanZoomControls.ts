import { Camera, OrthographicCamera, Vector2, Vector3 } from 'three';

const ZOOM_SPEED = 0.001;

export default class PanZoomControls {
    constructor(camera: OrthographicCamera, element: HTMLCanvasElement) {

        let dragging = false;
        const panStart = new Vector2();
        const panEnd = new Vector2();
        const panDelta = new Vector2();

        let frustrumWidth = camera.right - camera.left;
        let frustrumHeight = camera.bottom - camera.top;

        element.addEventListener('pointerdown', (evt) => {
            evt.preventDefault();
            dragging = true;
            panStart.set(evt.clientX, evt.clientY);
        });
        element.addEventListener('pointerup', (evt) => {
            evt.preventDefault()
            dragging = false;
        });
        element.addEventListener('pointermove', (evt) => {
            if (dragging) {
                panEnd.set(evt.clientX, evt.clientY);
                panDelta.subVectors(panEnd, panStart);

                camera.position.add(new Vector3(
                    -panDelta.x * frustrumWidth / window.innerWidth,
                    -panDelta.y * frustrumHeight / window.innerHeight,
                    0
                ));

                panStart.copy(panEnd);
            }
        });

        element.addEventListener('wheel', (evt) => {
            evt.preventDefault();
            const mutliplier = 1 + (evt.deltaY * ZOOM_SPEED);
            frustrumWidth *= mutliplier;
            frustrumHeight *= mutliplier;
            camera.left = -frustrumWidth / 2;
            camera.right = frustrumWidth / 2;
            camera.top = -frustrumHeight / 2;
            camera.bottom = frustrumHeight / 2;
            camera.updateProjectionMatrix();
        })
    }
}