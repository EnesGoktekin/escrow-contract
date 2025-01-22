
#![no_std]



multiversx_sc::imports!();

multiversx_sc::derive_imports!();


#[type_abi]
#[derive(

    multiversx_sc::codec::derive::TopEncode,
    multiversx_sc::codec::derive::TopDecode,
    

    multiversx_sc::codec::derive::NestedEncode, 
    multiversx_sc::codec::derive::NestedDecode,


    

    PartialEq,

    Eq,

    Debug,

    Clone
)]

pub enum OfferStatus {

    Active,

    Completed,

    Cancelled
}

#[type_abi]
#[derive(

    multiversx_sc::codec::derive::TopEncode,
    multiversx_sc::codec::derive::TopDecode,
    
    multiversx_sc::codec::derive::NestedEncode,
    multiversx_sc::codec::derive::NestedDecode,
 
    
    Clone
)]


pub struct Offer<M: ManagedTypeApi> {

    pub offer_id: u64,
    

    pub creator: ManagedAddress<M>,
    

    pub recipient: ManagedAddress<M>,

    pub amount: BigUint<M>,
    

    pub status: OfferStatus,
    

    pub created_timestamp: u64
}


#[multiversx_sc::contract]
pub trait EscrowContract {
 
    #[init]
    fn init(&self) {
        
        self.last_offer_id().set_if_empty(0u64);
    }

    #[payable("EGLD")]
    #[endpoint]
    fn create_offer(&self, buyer: ManagedAddress) {
        
        let payment = self.call_value().egld();
        
        require!(payment.clone_value() > BigUint::zero(), "Must pay more than 0.");

        require!(buyer != self.blockchain().get_caller(), "Cannot create offer for yourself.");
    
        let seller = self.blockchain().get_caller();
        
        let new_offer_id = self.last_offer_id().get() + 1;
        
        self.last_offer_id().set(new_offer_id);
    
        let offer = Offer {
            offer_id: new_offer_id,
            creator: seller.clone(), 
            recipient: buyer.clone(),
            amount: payment.clone_value(),
            status: OfferStatus::Active, 
            created_timestamp: self.blockchain().get_block_timestamp() 
        };
    
        self.offer(new_offer_id).set(offer.clone());
        
        self.user_offers(&seller).insert(new_offer_id);
        
        self.user_incoming_offers(&buyer).insert(new_offer_id);
    
        self.create_offer_event(new_offer_id, &seller, &buyer, &payment);
    }

    #[endpoint(cancelOffer)]
    fn cancel_offer(
        &self,
        offer_id: u64
    ) {
        let caller = self.blockchain().get_caller();
        
        let mut offer = self.offer(offer_id).get();
        require!(offer.status == OfferStatus::Active, "Offer not active");
        
        require!(offer.creator == caller, "Not offer creator");
    
        offer.status = OfferStatus::Cancelled;
        
        self.offer(offer_id).set(&offer);
    
        self.send().direct_egld(&caller, &offer.amount);
        
        self.cancel_offer_event(offer_id, &caller, &offer.amount);
    }

    #[endpoint(acceptOffer)]
    fn accept_offer(
        &self,
        offer_id: u64
    ) { // SCResult<()> return type'ı kaldırıldı
        let caller = self.blockchain().get_caller();
    
        let mut offer = self.offer(offer_id).get();
    
        require!(offer.recipient == caller, "Not offer recipient");
    
        offer.status = OfferStatus::Completed;
    
        self.offer(offer_id).set(&offer);
    
        self.send().direct_egld(&caller, &offer.amount);
    
        self.accept_offer_event(offer_id, &caller, &offer.amount);
    }

    
    #[view(getLastOfferId)]
    #[storage_mapper("lastOfferId")] 

    fn last_offer_id(&self) -> SingleValueMapper<u64>;

    #[view(getOffer)]
  
    #[storage_mapper("offer")]
   
    fn offer(&self, id: u64) -> SingleValueMapper<Offer<Self::Api>>;

 
    #[view(getUserOffers)]
  
    #[storage_mapper("userOffers")]
  
    fn user_offers(&self, user: &ManagedAddress) -> SetMapper<u64>;

   
    #[view(getUserIncomingOffers)]
  
    #[storage_mapper("userIncomingOffers")]
  
    fn user_incoming_offers(&self, user: &ManagedAddress) -> SetMapper<u64>;

    #[event("createOffer")]
 
    fn create_offer_event(
        &self,
        #[indexed] offer_id: u64,        // Teklifin benzersiz ID'si
        #[indexed] creator: &ManagedAddress,  // Teklifi oluşturan adres
        #[indexed] recipient: &ManagedAddress, // Alıcı adresi
        #[indexed] amount: &BigUint       // Teklif miktarı (EGLD)
    );

    
    #[event("cancelOffer")]
    fn cancel_offer_event(
        &self,
        #[indexed] offer_id: u64,        // İptal edilen teklifin ID'si
        #[indexed] creator: &ManagedAddress,  // Teklifi iptal eden (oluşturan) adres
        #[indexed] amount: &BigUint       // İade edilen miktar
    );

    
    #[event("acceptOffer")]
    fn accept_offer_event(
        &self,
        #[indexed] offer_id: u64,        // Kabul edilen teklifin ID'si
        #[indexed] recipient: &ManagedAddress, // Teklifi kabul eden (alıcı) adres
        #[indexed] amount: &BigUint       // Transfer edilen miktar
    );
    
    #[view(getActiveOffers)]
  

    fn get_active_offers(&self) -> MultiValueEncoded<Offer<Self::Api>> {
      
        let mut result = MultiValueEncoded::new();
        
        for offer_id in 1..=self.last_offer_id().get() {

            let offer = self.offer(offer_id).get();
            
            
            if offer.status == OfferStatus::Active {
                // push() ile listeye yeni eleman eklenir
                result.push(offer);
            }
        }
        
      
        result
    }

   
    #[view(getUserActiveOffers)]

    fn get_user_active_offers(
        &self,
        user: &ManagedAddress
    ) -> MultiValueEncoded<Offer<Self::Api>> {

        let mut result = MultiValueEncoded::new();
        
   
        for offer_id in self.user_offers(user).iter() {
       
            let offer = self.offer(offer_id).get();

            if offer.status == OfferStatus::Active {
               
                result.push(offer);
            }
        }
        

        result
    }
   
    #[view(getUserIncomingActiveOffers)]
    fn get_user_incoming_active_offers(
    
        &self,

        user: &ManagedAddress
    ) -> MultiValueEncoded<Offer<Self::Api>> {
       
        let mut result = MultiValueEncoded::new();
        
       
        for offer_id in self.user_incoming_offers(user).iter() {
      
            let offer = self.offer(offer_id).get();

       
            if offer.status == OfferStatus::Active {
            
                result.push(offer);
            }
        }
        



        result
    }
}