import { listen } from '@tauri-apps/api/event';
import './App.scss';

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { Scene } from './Scene';

type UpdateWorldPayload = {
  cells: ('Dead' | 'Alive')[][]
}

function App() {
  let [gridItems, setGridItems] = useState([[false]]);

  useEffect(() => {
    invoke('get_world_width').then(worldWidth => {
      const WORLD_WIDTH = worldWidth as any as number;
      setGridItems(
        Array.from({ length: WORLD_WIDTH })
          .map(() => Array.from({ length: WORLD_WIDTH })
            .map(() => false)));
    });
  }, []);
  
  useEffect(() => {
    listen('update-world', ({payload} : {payload: UpdateWorldPayload}) => {
      setGridItems(payload.cells.map(row => row.map(cell => cell === 'Alive')));
    });
  }, [gridItems]);
  
  return (
    <div className='App'>
      <Scene gridItems={gridItems} worldWidth={gridItems.length}/>
    </div>
  );
}

export default App;
