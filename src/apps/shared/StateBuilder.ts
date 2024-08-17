import { Action, ActionReducerMapBuilder, CaseReducer, PayloadAction, Slice } from '@reduxjs/toolkit';
import { cast, isStrictObject, prettify, TupleReduce } from 'readable-types';

export type SelectorsFor<T extends anyObject> = { [K in keyof T]: (state: T) => T[K] };
export type ReducersFor<T> = {
  [K in keyof T as `set${Capitalize<cast<K, string>>}`]: CaseReducer<T, PayloadAction<T[K]>>;
};

export const capitalize = (text: string) => {
  return text.slice(0, 1).toUpperCase() + text.slice(1);
};
export const matcher = (slice: Slice) => (action: Action) => action.type.startsWith(slice.name);

interface $GetState extends $<[acc: Record<string, any>, current: Slice]> {
  return: this[0] & { [x in this[1]['name']]: ReturnType<this[1]['getInitialState']> };
}

export type SelectorFor2<State extends anyObject, Current = State> = $if<isStrictObject<Current>, {
  then: ((state: State) => Current) & { [K in keyof Current]: SelectorFor2<State, Current[K]> };
  else: (state: State) => Current;
}>;

export class StateBuilder {
  static selectorsFor<T extends anyObject>(state: T): SelectorsFor<T> {
    const selectors = {} as SelectorsFor<T>;
    for (const key in state) {
      selectors[key] = (state: T) => state[key];
    }
    return selectors;
  }

  static reducersFor<T>(state: T): ReducersFor<T> {
    const reducers: any = {};
    for (const key in state) {
      reducers[`set${capitalize(key)}`] = (state: T, action: any) => {
        state[key] = action.payload;
      };
    }
    return reducers;
  }

  static addSliceAsExtraReducer(slice: Slice, builder: ActionReducerMapBuilder<{ [x in Slice['name']]: any }>) {
    builder.addMatcher(matcher(slice), (state, action) => {
      state[slice.name] = slice.reducer(state[slice.name], action);
    });
  }

  static compositeInitialState<T extends nLengthTuple<Slice>>(...slices: [...T]): prettify<TupleReduce<T, $GetState, {}>>;
  static compositeInitialState(...slices: Slice[]) {
    return slices.reduce((acc, slice) => {
      acc[slice.name] = slice.getInitialState();
      return acc;
    }, {} as anyObject);
  }

  static compositeSelector<T extends anyObject>(obj: T, selfSelector: any = (self: any) => self): SelectorFor2<T> {
    for (const key in obj) {
      selfSelector[key] = (state: any) => selfSelector(state)[key];

      if (typeof obj[key] === 'object' && !Array.isArray(obj[key])) {
        selfSelector[key] = StateBuilder.compositeSelector(obj[key], selfSelector[key]);
      }
    }
    return selfSelector;
  }

/*
  children: Slices = [] as any;

  constructor(public name: Name) {};

  appendChild<T extends Slice>(child: T): StateBuilder<Name, [...Slices, T]> {
    this.children.push(child);
    return this as any;
  }

  static addSlicesAsChildren(builder: ActionReducerMapBuilder<any>, slices: Slice[]) {
    slices.forEach((slice) => {
      builder.addMatcher(matcher(slice), (state, action) => {
        state[slice.name] = slice.reducer(state[slice.name], action);
      });
    });
  }

  build() {
    const slices = this.children;

    const initialState = {};

    return createSlice({
      name: this.name,
      initialState,
      reducers: reducersFor(initialState),
      selectors: selectorsFor(initialState),
      extraReducers(builder) {
        StateBuilder.addSlicesAsChildren(builder, slices);
      },
    });
  }; */
}
