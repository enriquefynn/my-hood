// This file contains type definitions for your data.
// It describes the shape of the data, and what data type each property should accept.
export type User = {
  id: string;
  name: string;
  birthday: string;
  address: string;
  activity?: string;
  email?: string;
  personal_phone?: string;
  commercial_phone?: string;
  uses_whatsapp: boolean;
  identities?: string;
  profile_url?: string;
};

export type Association = {
  id: string;
  name: string;
  neighborhood: string;
  country: string;
  state: string;
  address: string;
  identity?: string;
};

export type UserAssociation = {
  user_id: string;
  association_id: string;
};

export type AssociationAdmin = {
  user_id: string;
  association_id: string;
};

export type AssociationTreasurer = {
  user_id: string;
  association_id: string;
  start_date: string;
  end_date?: string;
};

export type Transaction = {
  id: string;
  association_id: string;
  creator_id: string;
  details: string;
  amount: number;
  reference_date: string;
  deleted: boolean;
};