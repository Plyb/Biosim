import { listen } from '@tauri-apps/api/event';
import './App.scss';

import { useState } from 'react';

type UpdateWorldPayload = {
  cells: ('Dead' | 'Alive')[][]
}

function App() {
  let [gridItems, setGridItems] = useState(Array.from({ length: 32 })
    .map(() => Array.from({ length: 32 })
        .map(() => true)));
    
  listen('update-world', ({payload} : {payload: UpdateWorldPayload}) => {
    setGridItems(payload.cells.map(row => row.map(cell => cell === 'Alive')));
  });
  
  return (
    <div className='App'>
      <div className='grid'>
        {gridItems.map(row =>
          <tr>
            {row.map(cell => <td className={cell ? 'filled' : 'empty'}></td>)}
          </tr>)}
      </div>
    </div>
  );
}

export default App;
