import { createWrapperComponent } from '../utils';

// MenuBarExtra is used for menu bar commands  
// Since we don't have a menu bar in Linux, we'll create a stub component
const MenuBarExtraBase = createWrapperComponent('MenuBarExtra');

// MenuBarExtra sub-components
const Item = createWrapperComponent('MenuBarExtra.Item');
const Section = createWrapperComponent('MenuBarExtra.Section');
const Separator = createWrapperComponent('MenuBarExtra.Separator');
const Submenu = createWrapperComponent('MenuBarExtra.Submenu');

// Attach sub-components
export const MenuBarExtra = Object.assign(MenuBarExtraBase, {
    Item,
    Section,
    Separator,
    Submenu
});
